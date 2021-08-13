use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

//////////////////////////////////////////
//// MODE 0 -> INIT                   ////
//// MODE 1 -> TOP SEPARATOR Detected ////
//// MODE 2 -> END SEPARATOR Detected ////
//// MODE 3 -> READ CLIENT DATA       ////
//////////////////////////////////////////

static PATH: &str = "./_GJIL.doc";
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
    println!("\n|‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|\n|  GJIL SPLITTER Output  |\n|________________________|\n");
    let mut mode: i32 = 0;
    let mut top_separator = String::new();
    let mut end_separator = String::new();
    let mut client: Vec<String> = Vec::new();
    let mut tmp: Vec<String> = Vec::new();
    let mut i: usize = 0;
    let mut identifier = String::new();
    if let Ok(lines) = read_lines(&PATH) {
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
                       println!("{}", top_separator);
                       continue;
                   }
                   else if mode == 2 {
                       end_separator.push_str(&format!("{}{}",ip,"\n"));
                       println!("{}{}", "\n",end_separator);
                       // OUTPUT FILES
                       let mut j: i32 = 0;
                       for client in client.iter() {
                           match write(j, format!("{}{}{}", top_separator,client,end_separator)) {
                               Err(e) => println!("{:?}", e),
                               _ => ()
                           }
                           j += 1;
                       }
                       if j > 0 {
                           match del("./_GJIL.doc") {
                               Err(e) => println!("{:?}", e),
                               _ => ()
                           }
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
                                       println!("{}{}{}{}", "New Client Detected: ",detected_id, "  PAGE: ", &i[pos2+DATA[k][2]..pos2+DATA[k][2]+DATA[k][3]]); // Extract the Page number and print it
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
           }
       }
   } else {
       println!("An error has occured");
   }
}

fn write(j: i32, output: String) -> std::io::Result<()> {
    let mut file = File::create(format!("{}{}{}", "./GJIL_TR_",j,".doc"))?;
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
