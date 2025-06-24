use crossterm::{
    cursor::{Hide , MoveTo , Show},
    event::{read , Event , KeyCode , KeyEvent , KeyModifiers},
    execute, 
    terminal::{disable_raw_mode , enable_raw_mode , Clear , ClearType , EnterAlternateScreen , LeaveAlternateScreen}
};
use std::{fs::File, io::{self , BufRead , BufReader , Stdout , Write} ,  process::exit};

enum EditorMode {
    Normal , 
    Insert
}

fn read_file_to_buffer(filename: &str) -> io::Result<Vec<String>> {
    let file = match File::open(filename) {
        Ok(f) => f , 
        Err(_) => {
            File::create(filename)?;
            File::open(filename)?
        } ,
    };
    let reader =  BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).collect())
}

fn write_buffer(filename:&str , buffer :&[String] ) -> std::io::Result<()>{
    let mut file = File::create(filename)?;
    for line in buffer {
        writeln!(file , "{}" , line)?;

    }
    Ok(())
}


fn draw(stdout: &mut Stdout, buffer: &[String], cursor_x: usize, cursor_y: usize , mode :  &EditorMode  , status_message : Option<&str>) -> std::io::Result<()> {

    execute!(stdout , Clear(ClearType::All))?;
    for (i , line)  in buffer.iter().enumerate() {
        execute!(stdout , MoveTo(0 , i as u16))?;
        print!("{}" , line);
    }
    execute!(
        stdout,
        MoveTo(0, buffer.len() as u16 + 1),
    )?;
    match mode {
        EditorMode::Insert => print!("-- INSERT --"),
        EditorMode::Normal => print!("-- NORMAL -- (press 'i' to insert, 'q' to quit)"),
    }
    let safe_y = cursor_y.min(buffer.len().saturating_sub(1));
    let safe_x = cursor_x.min(buffer.get(safe_y).map_or(0 , |l| l.len()));
    if let Some(m) = status_message {
        execute!(stdout , MoveTo(0 , buffer.len() as u16 + 2))?;
        writeln!(stdout , "{}" , m)?;
    }

    execute!(stdout , MoveTo(safe_x as u16  , safe_y as u16))?;
    stdout.flush()?;
    Ok(())
    
}
fn main() -> std::io::Result<()> {
    let mut mode = EditorMode::Normal;
    let mut fname = String::new();
    let mut status_message = None;

    io::stdin().read_line(&mut fname).expect("Failed to read line name");
    let fname = fname.trim();

    let mut buffer = read_file_to_buffer(fname).unwrap_or_else(|_| {
        println!("Error Reading File. Please verify it exists.");
        exit(1);
    });

    if buffer.is_empty() {
        buffer.push(String::new());
    }

    let (mut cursor_x, mut cursor_y) = (0, 0);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    loop {
        draw(&mut stdout, &buffer, cursor_x, cursor_y, &mode, status_message.as_deref())?;

        if let Event::Key(KeyEvent { code, modifiers, .. }) = read()? {
            match mode {
                EditorMode::Normal => match (code, modifiers) {
                    (KeyCode::Char('q'), _) => break,
                    (KeyCode::Char('i'), _) => mode = EditorMode::Insert,
                    (KeyCode::Up, _) => {
                        if cursor_y > 0 {
                            cursor_y -= 1;
                            cursor_x = cursor_x.min(buffer[cursor_y].len());
                        }
                    }
                    (KeyCode::Down, _) => {
                        if cursor_y + 1 < buffer.len() {
                            cursor_y += 1;
                            cursor_x = cursor_x.min(buffer[cursor_y].len());
                        }
                    }
                    (KeyCode::Left, _) => {
                        if cursor_x > 0 {
                            cursor_x -= 1;
                        }
                    }
                    (KeyCode::Right, _) => {
                        if cursor_x < buffer.get(cursor_y).map_or(0, |l| l.len()) {
                            cursor_x += 1;
                        }
                    }
                    _ => {}
                },

                EditorMode::Insert => match (code, modifiers) {
                    (KeyCode::Char('s'), m) if m.contains(KeyModifiers::CONTROL) => {
                        if let Err(_) = write_buffer(fname, &buffer) {
                            status_message = Some("Error saving file".to_string());
                        } else {
                            status_message = Some("File Saved".to_string());
                        }
                    }
                    (KeyCode::Esc, _) => mode = EditorMode::Normal,
                    (KeyCode::Char(c), _) => {
                        if let Some(line) = buffer.get_mut(cursor_y) {
                            if cursor_x <= line.len() {
                                line.insert(cursor_x, c);
                                cursor_x += 1;
                            }
                        }
                    }
                    (KeyCode::Backspace, _) => {
                        if let Some(line) = buffer.get_mut(cursor_y) {
                            if cursor_x > 0 && cursor_x <= line.len() {
                                line.remove(cursor_x - 1);
                                cursor_x -= 1;
                            }
                        }
                    }

                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;
    Ok(())
}
