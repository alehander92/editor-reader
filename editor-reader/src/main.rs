use std::env;
use std::ffi::CString;
use std::path::PathBuf;
use std::fs;
use futures::executor::block_on;
use tree_sitter::{Parser, Language, Tree, Node, TreeCursor};
// use async_std::fs::File;
// use async_std::prelude::*;
use std::str::FromStr;
use std::process::Command;
// use std::os::unix::process::CommandExt;
// use std::io::stdin;
// use std::ffi::OsStr;
use std::time::Duration;
use std::thread::sleep;

extern "C" { fn tree_sitter_java() -> Language; }

#[derive(Debug, Clone)]
enum Event { 
   Up,
   Down,
   Left,
   Right,
   Describe
}

#[derive(Debug, Clone)]
struct Message {
   event: Event,
   line: usize,
   column: usize
}

#[derive(Debug, Clone)]
enum Description {
  Text(String)
}

impl FromStr for Event {

    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<Event, Self::Err> {
        match input {
            "up"  => Ok(Event::Up),
            "down"  => Ok(Event::Down),
            "left"  => Ok(Event::Left),
            "right" => Ok(Event::Right),
            "describe" => Ok(Event::Describe),
            // this `Err` is rust builtin Err, not self::Err
            _      => Err(std::io::Error::new(std::io::ErrorKind::Other, "can't parse event")),
        }
    }
}

fn ensure_support(path: String) -> Result<(), std::io::Error> {
   let pipe_path = PathBuf::from(format!("/tmp/editor-reader{}.pipe", path.clone()));
   let tmp_editor_reader_dir = PathBuf::from("/tmp/editor-reader");
   fs::create_dir_all(tmp_editor_reader_dir)?;
   if !pipe_path.exists() {
      let pipe_path_str = CString::new(
         format!(
            "/tmp/editor-reader{}.pipe",
            path))?;
      if let Some(parent) = PathBuf::from(path.clone()).parent() {
         let parent_path = PathBuf::from(format!("/tmp/editor-reader/{}", parent.display()));
         fs::create_dir_all(parent_path)?;         
         println!("pipe path {:?}", pipe_path_str);
         unsafe {
            libc::mkfifo(pipe_path_str.as_ptr() as *const i8, 0644);
         }
      } else {
         println!("can't create parent path of {}", pipe_path.display());
      }
      println!("path {:?}", path);
   }
   Ok(())
}


fn parse_file_initial(path: String) -> Result<(Parser, Tree, String), std::io::Error> {
   let mut parser = Parser::new();
   let language = unsafe { tree_sitter_java() };
   parser.set_language(language).unwrap();
   let source = fs::read_to_string(path)?;
   if let Some(tree) = parser.parse(source.clone(), None) {
      Ok((parser, tree, source))
   } else {
      Err(std::io::Error::new(std::io::ErrorKind::Other, "tree is none"))
   }  
}

fn parse_message(text: String) -> Result<Message, std::io::Error> {
  //  println!("{:?}", text);
   let tokens: Vec<&str> = text.split_whitespace().collect();
  //  println!("{:?}", tokens);
   if tokens.len() != 3 {
      return Err(std::io::Error::new(std::io::ErrorKind::Other, "expected 3 tokens"))   
   }
   let event: Event = tokens[0].parse()?;
   // TODO thiserror checking
   let line_result: Result<usize, _> = tokens[1].parse();
   if let Ok(line) = line_result {
      let column_result: Result<usize, _> = tokens[2].parse();
      if let Ok(column) = column_result {
         let msg = Message { event: event, line: line, column: column};
         return Ok(msg);
      }
   }
   // Message { }
   Err(std::io::Error::new(std::io::ErrorKind::Other, "expected integers for line and column, but couldn't parse them"))
   // Ok())
}

async fn read_event_message(pipe_path: PathBuf) -> Result<Message, std::io::Error> {   
   // let mut file = File::open(pipe_path).await?;
   println!("wait");
   // println!("{:?}", pipe_path);
   let text = async_std::fs::read_to_string(pipe_path).await?;
   println!("text {:?}", text);
   parse_message(text)
}

fn node_find_by_position<'a>(node: Node<'a>, line: usize, column: usize, cursor: &mut TreeCursor<'a>) -> Option<Node<'a>> {
  println!("node {:?}", node);
  println!("location {} _ {} in {} {} _ {} {} ?", 
    line, column,
    node.start_position().row + 1, node.start_position().column + 1,
    node.end_position().row + 1, node.end_position().column + 1);
  // if node.start_position().row <= line && node.end_position().row >= line &&
  if (node.start_position().row + 1 == line && node.start_position().column + 1 <= column || 
     node.start_position().row + 1 < line) &&
     (node.end_position().row + 1 == line && node.end_position().column + 1 >= column ||
     node.end_position().row + 1 > line) {
    if node.child_count() == 0 {
      return Some(node)
    } else {
      
      let node_children: Vec<Node> = node.children(cursor).collect();

      for child in node_children {
        println!("child {:?}", child);
        let result = node_find_by_position(child, line, column, cursor);
        if let Some(result_node) = result {
          return result;
        }
      }
    }
  }
  // println!("{:?}", root.child(0)?);
  None
}
fn find_by_position<'a>(tree: &'a Tree, line: usize, column: usize) -> Option<Node<'a>> {
  let root = tree.root_node();
  let mut cursor = root.walk();
  let node = node_find_by_position(root, line, column, &mut cursor);
  println!("node {:?}", node);
  node
}

fn generate_descriptions(node: Node, source_lines: Vec<String>) -> Vec<Description> {
  // generate a description for each node
  
  match node.kind() {
    "class" => {
      vec![Description::Text("class".to_string())]
    },
    "identifier" => {
      // we expect start row == end row
      vec![
        Description::Text(
          source_lines[node.start_position().row][node.start_position().column .. node.end_position().column].to_string())]
    },
    _ => {
      vec![]
    }
  }
}


struct EspeakBackend {

}

trait Backend {
  type BackendDescriptions;

  fn generate(&self, descriptions: Vec<Description>) -> Self::BackendDescriptions;
  fn process(&self, descriptions: Self::BackendDescriptions) -> Result<(), std::io::Error>;
}

impl EspeakBackend {
  fn generate_description(&self, description: Description) -> String {
    match description {
      Description::Text(text) => {
        text
      }
    }
  }
}

impl Backend for EspeakBackend {
  type BackendDescriptions = String;

  fn generate(&self, descriptions: Vec<Description>) -> String {
    let mut text = "".to_string();
    for description in descriptions {
      text += &self.generate_description(description);
    }
    text
  }

  fn process(&self, descriptions: String) -> Result<(), std::io::Error> {
    let espeak_shell_lang = "en-uk".to_string();
    // let mut command = 
    println!("espeak");
    println!("descriptions {}", descriptions);
    Command::new("espeak")
      .arg("-v").arg(&espeak_shell_lang)
      .arg(&descriptions)
      .spawn()?
      .wait()?;
    Ok(())
  }
}



fn process_message(tree: Tree, message: Message, source_lines: Vec<String>) -> Result<(), std::io::Error> {
  println!("message {:?}", message);
  match message.event {
    Event::Describe => { // TODO low level vs high level
      if let Some(node) = find_by_position(&tree, message.line, message.column) {
        let descriptions = generate_descriptions(node, source_lines);
        println!("describe {:?}", descriptions);
        let backend = EspeakBackend {};
        let backend_descriptions = backend.generate(descriptions);
        println!("backend_descriptions {:?}", backend_descriptions);
        backend.process(backend_descriptions)?;
      }
    },
    _ => {
      ()
    }
  }
  Ok(())
}

async fn process(path: String) -> Result<(), std::io::Error> {
   println!("process path {:?}", path);
   let pipe_path = PathBuf::from(format!("/tmp/editor-reader/{}.pipe", path));
   let (_parser, tree, source) = parse_file_initial(path)?;
   // println!("{:?}", tree);
   println!("tree node {:?}", tree.root_node().to_sexp());
   loop {
      if let Ok(message) = read_event_message(pipe_path.clone()).await {
         process_message(tree.clone(), message, source.split("\n").map(|line| line.to_string()).collect())?;
      }
      sleep(Duration::from_millis(200));
   }
}


fn main() -> Result<(), std::io::Error> {
   let length = env::args().len();
   // TODO loop many:
   // editor-reader : starts the process waiting for all files
   // editor-reader <path> : ensures pipe file/register for path

   // now
   // editor-reader <path> --wait : starts the process, but only for this path
   //   and 
   // editor-reader <path> : ensures pipe file for path exists

   // pipe files are in /tmp/editor-reader/<path>
   // and the process should monitor them eventually TODO

   if length == 2 {
       let path = env::args().nth(1).unwrap();
       ensure_support(path)?;
   } else if length == 3 {
       let path = env::args().nth(1).unwrap();
       let arg = env::args().nth(2).unwrap();
       // TODO: ensure pipe exists here?
       if arg == "--wait" {
         block_on(process(path))?;
       }
   }
   Ok(())
}
