use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::Command;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

//////////////////////////////////////////
//// MODE 0 -> INIT                   ////
//// MODE 1 -> TOP SEPARATOR Detected ////
//// MODE 2 -> END SEPARATOR Detected ////
//// MODE 3 -> READ CLIENT DATA       ////
//////////////////////////////////////////

static PATH: &str = "/_GJIL.doc";
static MARKER: [&str; 3] = ["NOD   :", "CHEF DE SERVICE :", "AFFECTATION          :"];
static PAGE: [&str; 3] = ["PAGE", "FEUILLET NO:", "FICHE  :"];
static POP_STOP: [&str; 3] = ["PAGE ", "REF ", "REF "];
static STOP_TYPE: [&str; 3] = ["contain", "contain", "contain"];
static DATA: [[usize; 4]; 3] = [
    [MARKER[0].len()+1,6,PAGE[0].len(),5], // [Marker length+1, identifier length, Page marker length, Page Number Length]
    [MARKER[1].len()+1,6,PAGE[1].len(),6],
    [MARKER[2].len()+1+4,9,PAGE[2].len(),7] // The first 4 chars of the AFFECTATION number are not used to identify a Client
];

fn main() {
    reset_log();
    log("\n|‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|\n|  GJIL SPLITTER Output  |\n|________________________|\n");
    let mut mode: i32 = 0;
    let mut top_separator = String::new();
    let mut end_separator = String::new();
    let mut client: Vec<String> = Vec::new();
    let mut tmp: Vec<String> = Vec::new();
    let mut i: usize = 0;
    let mut identifier = String::new();

    let current_path = get_current_path(); // Get the path of the currently running executable
    let file_path = get_path(&current_path); // [Input file, Output file] as configure in the excel
    let mut blacklist: Vec<String> = Vec::new(); // A Vec of failed macro

    if let Ok(lines) = read_lines(&format!("{}{}", file_path[0], PATH)) {
       for line in lines {
           if let Ok(ip) = line {
               if ip.contains("TOP SEPARATOR") {
                   mode = 1;
               }
               if ip.contains("END SEPARATOR") {
                   client.push(String::new());
                   for line in tmp.iter() {
                       client[i-1].push_str(&format!("{}",line));
                   }
                   mode = 2;
               }
               if ip.contains("********************************************************************") {
                   if mode == 1 {
                       top_separator.push_str(&format!("{}{}",ip,"\n"));
                       mode = 3;
                       log(&format!("{}", top_separator));
                       continue;
                   }
                   else if mode == 2 {
                       end_separator.push_str(&format!("{}{}",ip,"\n"));
                       log(&format!("{}{}", "\n",end_separator));
                       // OUTPUT FILES
                       let mut j: i32 = 0;
                       for client in client.iter() {
                           match write(j, format!("{}\n{}\n{}", top_separator,client,end_separator), &file_path[0]) {
                               Err(e) => log(&format!("{:?}", e)),
                               _ => ()
                           }
                           j += 1;
                       }
                       if j > 0 {
                           match del(&format!("{}/_GJIL.doc", file_path[0])) {
                               Err(e) => log(&format!("{:?}", e)),
                               _ => ()
                           }
                           log("Renaming...");
                           run_vbs("rename", &format!("{}{}{}",'"',current_path,'"'), "", ""); // Running the renaming macro
                           log("Applying macro to :");
                           let list = visit_dirs(Path::new(&file_path[1])); // Get all the file in the input dir
                           if let Ok(list) = list {
                               for path in list.iter() {
                                   log(&format!("\n{}", path));
                                   let name = path.split("\\");            // Only keep
                                   let mut tmp: Vec<String> = Vec::new();  // the name
                                   for i in name {                         // of the
                                       tmp.push(i.to_string());            // file
                                   }
                                   let name = tmp.last().unwrap().replace(".doc", ""); // remove the extension
                                   let mut tmp_name: String = String::new();
                                   let mut success: bool = false;
                                   for i in name.replace("-", "_").split("_") { // Iterate through each part of the name to find the corresponding macro (CTl_RSP_OP : CTL -> CTL_RSP -> CTL_RSP_OP)
                                       tmp_name.push_str(i);
                                       if !blacklist.contains(&tmp_name.to_string()) {
                                           log(&format!("trying: {}", tmp_name));
                                           run_vbs("macro", &format!("{}{}{}",'"',path,'"'), &tmp_name, &format!("{}{}{}",'"',current_path,'"'));
                                           if get_macro_result() {
                                               log(&format!("Macro successfully found ({}) and apply to {}.doc", tmp_name, name));
                                               success = true;
                                               break;
                                           } else {
                                               blacklist.push(tmp_name.to_string());
                                           }
                                       }
                                       tmp_name.push_str("_");
                                   }
                                   if success {
                                       // Rename the file in the output
                                   } else {
                                       log(&format!("Any macro found for {}.doc", name));
                                   }
                               }
                           }
                           match del("./log_macro.txt") {
                               Err(e) => log(&format!("{:?}", e)),
                               _ => ()
                           }
                           match del("./config.txt") {
                               Err(e) => log(&format!("{:?}", e)),
                               _ => ()
                           }
                       } else {
                           log("No client found in the _GJIL.doc file");
                       }
                   }
               }
               if mode == 0 {
                   continue;
               }
               else if mode == 1 {
                   top_separator.push_str(&format!("{}{}",ip,"\n"));
               }
               else if mode == 2 {
                   end_separator.push_str(&format!("{}{}",ip,"\n"));
               }
               else { // mode == 3
                   tmp.push(format!("{}{}",ip,"\n"));
                   for k in 0..MARKER.len() {
                       if ip.contains(MARKER[k]) { // If a marker is detected
                           let pos = (ip.find(MARKER[k])).unwrap(); // pos keep the index of the marker
                           let detected_id = &ip[pos+DATA[k][0]..pos+DATA[k][0]+DATA[k][1]]; // From which the client identifier is deduce
                           if format!("{}", detected_id) != format!("{}", identifier) { // Check if this client is the same as the previous one
                               identifier = detected_id.to_string(); // New client detected -> save it as the current one
                               let mut buff: Vec<String> = Vec::new();
                               if STOP_TYPE[k] == "contain" {
                                   while !tmp.last().unwrap().contains(POP_STOP[k]) {
                                       buff.push(tmp.pop().unwrap());
                                   }
                               }
                               buff.push(tmp.pop().unwrap()); // The last line wasn't transfert due to the "while"
                               for i in buff.iter() {
                                   if i.contains(PAGE[k]) { // get the line where the page marker is
                                       let pos2 = (i.find(PAGE[k])).unwrap(); // ges the index of the page marker
                                       log(&format!("{}{}{}{}", "New Client Detected: ",detected_id, "  PAGE: ", &i[pos2+DATA[k][2]..pos2+DATA[k][2]+DATA[k][3]])); // Extract the Page number and print it
                                       break;
                                   }
                               }
                               if i != 0 {
                                   client.push(String::new());
                                   for line in tmp.iter() {
                                       client[i-1].push_str(&format!("{}",line)); // Save What remaining in Tmp as a new client
                                   }
                               }
                               i += 1;
                               tmp = Vec::new();
                               for _l in 0..buff.len() {
                                   tmp.push(buff.pop().unwrap().to_string()); // Put back the line which was remove to the new client tmp vec
                               }
                           }
                       }
                   }
               }
           } else {
               log("An error has occured while reading a line");
               run_vbs("error", "", "", "");
               std::process::exit(1);
           }
       }
   } else {
       log("An error has occured while reading the file");
       run_vbs("error", "", "", "");
       std::process::exit(1);
   }
   run_vbs("end", "", "", "");
}

fn write(j: i32, output: String, file_path: &String) -> std::io::Result<()> {
    let mut file = File::create(format!("{}{}{}{}", file_path, "/GJIL_TR_",j,".doc"))?;
    file.write_all(output.as_bytes())?;
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<encoding_rs_io::DecodeReaderBytes<File, Vec<u8>>>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(file)).lines())
}

fn del(file: &str) -> std::io::Result<()> {
    std::fs::remove_file(file)?;
    Ok(())
}

fn get_path(current_path: &String) -> [String; 3] {
    run_vbs("getPath", &format!("{}{}{}",'"',current_path,'"'), "", "");
    let contents = std::fs::read_to_string("./config.txt")
        .expect("An error has occured while reading the config file");
    let split = contents.split("|SEP|");
    let mut path: Vec<String> = Vec::new();
    for i in split {
        path.push(i.to_string().replace("\n", "").replace("\r", ""));
    }
    return [path[1].to_string(), path[2].to_string(), path[3].to_string()]
}

fn get_macro_result() -> bool {
    let contents = std::fs::read_to_string("./log_macro.txt")
        .expect("An error has occured while reading the config file");
    if contents.trim() == "SUCCESS" {
        return true;
    } else {
        return false;
    }
}

fn get_current_path() -> String {
    match std::env::current_exe() {
        Ok(v) => {
            let result = format!("{:?}" ,v).replace("gjil_sep.exe","").replace('"', "");
            return result;
        },
        Err(_e) => {
            return String::from("fail")
        }
    }
}

fn run_vbs(arg0: &str, arg1: &str, arg2: &str, arg3: &str) {
    let path = get_current_path();
    let _output = Command::new("cmd")
                    .args(&["/C", &format!("{}\\gjil_sep.vbs {} {} {} {}", path, arg0, arg1, arg2, arg3)])
                    .output()
                    .expect("failed to execute process");
}

fn visit_dirs(dir: &Path) -> io::Result<Vec<String>> {
    let mut doc_list: Vec<String> = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                let path = format!("{:?}", path).replace('"', "");
                doc_list.push(path)
            }
        }
    }
    Ok(doc_list)
}

fn log(data: &str) {
    println!("{}", data);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("./log.txt")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", data) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn reset_log() {
    let _file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("./log.txt");
}
