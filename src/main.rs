use crossterm::{
    cursor::{Hide , MoveTo , Show},
    event::{read , Event , KeyCode , KeyEvent},
    execute, 
    terminal::{disable_raw_mode , enable_raw_mode , Clear , ClearType , EnterAlternateScreen , LeaveAlternateScreen}
};
use std::{fs::File, io::{self , BufRead , BufReader , Stdout , Write}, os::unix::fs::lchown, process::exit};

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

fn draw(stdout: &mut Stdout, buffer: &[String], cursor_x: usize, cursor_y: usize , mode :  &EditorMode ) -> std::io::Result<()> {

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
    execute!(stdout , MoveTo(safe_x as u16  , safe_y as u16))?;
    stdout.flush()?;
    Ok(())
    
}

fn main() -> std::io::Result<()> {
    let mut mode = EditorMode::Normal;
    let mut fname = String::new();
    io::stdin().read_line(&mut fname).expect("Failed to read line name");
    let fname = fname.trim();
    let mut buffer = read_file_to_buffer(fname).unwrap_or_else(|e| {
        println!("Error Reading File Please Verify it Exists");
        exit(1);
    });
    let (mut cursor_x, mut cursor_y) = (0, 0);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout , EnterAlternateScreen , Hide)?;

 loop {
    draw(&mut stdout, &buffer, cursor_x, cursor_y , &mode)?;

    if let Event::Key(KeyEvent { code, .. }) = read()? {
        match mode {
            EditorMode::Normal => match code {
                KeyCode::Char('q') => break,
                KeyCode::Char('i') => mode = EditorMode::Insert,
                KeyCode::Up => {
                    if cursor_y > 0 {
                        cursor_y -= 1;
                        cursor_x = cursor_x.min(buffer[cursor_y].len());
                    }
                }
                KeyCode::Down => {
                    if cursor_y + 1 < buffer.len() {
                        cursor_y += 1;
                        cursor_x = cursor_x.min(buffer[cursor_y].len());
                    }
                }
                KeyCode::Left => {
                    if cursor_x > 0 {
                        cursor_x -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_x < buffer.get(cursor_y).map_or(0, |l| l.len()) {
                        cursor_x += 1;
                    }
                }
                _ => {}
            },

            EditorMode::Insert => match code {
                KeyCode::Esc => mode = EditorMode::Normal,
                KeyCode::Char(c) => {
                    if let Some(line) = buffer.get_mut(cursor_y) {
                        if cursor_x <= line.len() {
                            line.insert(cursor_x, c);
                            cursor_x += 1;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(line) = buffer.get_mut(cursor_y) {
                        if cursor_x > 0 && cursor_x <= line.len() {
                            line.remove(cursor_x - 1);
                            cursor_x -= 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

    disable_raw_mode()?;
    execute!(stdout ,  Show , LeaveAlternateScreen)?;
    Ok(())
}