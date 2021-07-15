use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

//////////////////////////////////////////
//// MODE 0 -> INIT                   ////
//// MODE 1 -> TOP SEPARATOR Detected ////
//// MODE 2 -> END SEPARATOR Detected ////
//// MODE 3 -> READ CLIENT DATA       ////
//////////////////////////////////////////

static PATH: &str = "./_GJIL.doc";
static MARKER: [&str; 3] = ["NOD   :", "CHEF DE SERVICE :", "AFFECTATION          :"];
static PAGE: [&str; 3] = ["PAGE", "FEUILLET NO:", "FICHE  :"];
static DATA: [[usize; 6]; 3] = [
    [MARKER[0].len()+1,6,8,PAGE[0].len(),5,7], // [Marker length+1, identifier length, Number of line to pop and push, Page marker length, Page Number Length, Number of line to pop to find the page marker]
    [MARKER[1].len()+1,6,13,PAGE[1].len(),6,2],
    [MARKER[2].len()+1+4,9,10,PAGE[2].len(),7,6] // The first 4 chars of the AFFECTATION number are not used to identify a Client
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
                               for _l in 0..DATA[k][2] {
                                   buff.push(tmp.pop().unwrap()); // Store and remove the lines related to the new client which are still in the previous one Vec's
                               }
                               let page_ln = &buff[DATA[k][5]]; // get the line where the page marker is
                               let pos2 = (page_ln.find(PAGE[k])).unwrap(); // ges the index of the page marker
                               println!("{}{}{}{}", "New Client Detected: ",detected_id, "  PAGE: ", &page_ln[pos2+DATA[k][3]..pos2+DATA[k][3]+DATA[k][4]]); // Extract the Page number and print it
                               if i != 0 {
                                   client.push(String::new());
                                   for line in tmp.iter() {
                                       client[i-1].push_str(&format!("{}",line)); // Save What remaining in Tmp as a new client
                                   }
                               }
                               i += 1;
                               tmp = Vec::new();
                               for _l in 0..DATA[k][2] {
                                   tmp.push(buff.pop().unwrap().to_string()); // Put back the line which was remove to the new client tmp vec
                               }
                           }
                       }
                   }
               }
           }
       }
   }
}

fn write(j: i32, output: String) -> std::io::Result<()> {
    let mut file = File::create(format!("{}{}{}", "./GJIL_TR_",j,".doc"))?;
    file.write_all(output.as_bytes())?;
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
