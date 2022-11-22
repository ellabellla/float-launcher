use std::{process::{exit, Command}, fs, path::PathBuf, io::{Write, self, Seek}, time::Duration};

use clap::{Parser, Subcommand};
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute, event::{EnableMouseCapture, Event, self, KeyCode, DisableMouseCapture}};
use serde::{Deserialize, Serialize};
use file_lock::{FileLock, FileOptions};
use tui::{backend::{CrosstermBackend, Backend}, Terminal, Frame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{Block, Borders, ListState, ListItem, Paragraph, List}, text::{Spans, Span}, style::{Style, Modifier}};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Config Directory
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Add a command to the launcher
    Add{
        /// Name
        name: String,

        /// Description
        description: String, 

        /// Command
        command: String,

        /// Tags
        tags: Vec<String>
    },
    /// Remove a command from the launcher
    Remove{
        /// Name
        name: String
    },
    /// Open the launcher (running with no subcommands will also open the launcher)
    Launch,
}


#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    /// Name
    pub name: String,

    /// Description
    description: String, 
    
    /// Command
    pub command: String,

    /// Tags
    pub tags: Vec<String>
}

struct StatefulList{
    state: ListState,
}

impl StatefulList {
    fn new() -> StatefulList {
        StatefulList { state: ListState::default() }
    }

    fn first(&mut self, max: usize) {
        if max == 0 {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    fn selected(&mut self) -> Option<usize> {
        self.state.selected()
    }
    
    fn next(&mut self, max: usize) {
        if max == 0 {
            self.state.select(None);
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= max - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn previous(&mut self, max: usize) {
        if max == 0 {
            self.state.select(None);
        } else {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        max - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

fn add_to_database(database_path: &PathBuf, name: String, description: String, command: String, tags: Vec<String>) -> !{
    let mut lock = FileLock::lock(database_path, true, 
        FileOptions::new()
            .read(true)
            .write(true)
        ).unwrap_or_else(|_| {
        println!("Couldn't open database file");
        exit(1);
    });

    let mut database: Vec<Entry> = serde_json::from_reader(&lock.file).unwrap_or_else(|_| {
        println!("Couldn't read database file");
        exit(1);
    });

    lock.file.seek(io::SeekFrom::Start(0)).unwrap_or_else(|_| {
        println!("Couldn't read database file");
        exit(1);
    });

    for entry in &database {
        if entry.name == name {
            println!("Name already exists in database");
            exit(1);
        }
    }

    database.push(Entry{name, description, command, tags});

    serde_json::to_writer_pretty(&lock.file, &database).unwrap_or_else(|_| {
        println!("Couldn't write to database file");
        exit(1);
    });

    exit(0);
}

fn remove_from_database(database_path: &PathBuf, name: &str) -> ! {
    let mut lock = FileLock::lock(database_path, true, 
        FileOptions::new()
            .read(true)
            .write(true)
    ).unwrap_or_else(|_| {
        println!("Couldn't open database file");
        exit(1);
    });

    let mut database: Vec<Entry> = serde_json::from_reader(&lock.file).unwrap_or_else(|_| {
        println!("Couldn't read database file");
        exit(1);
    });

    lock.file.seek(io::SeekFrom::Start(0)).unwrap_or_else(|_| {
        println!("Couldn't read database file");
        exit(1);
    });

    for i in 0..database.len() {
        if database[i].name == name {
            database.remove(i);

            let json = serde_json::to_string_pretty(&database).unwrap_or_else(|_| {
                println!("Couldn't write to database file");
                exit(1);
            });
        
            lock.file.set_len(json.as_bytes().len() as u64).unwrap_or_else(|_| {
                println!("Couldn't write to database file");
                exit(1);
            });
            lock.file.write_all(json.as_bytes()).unwrap_or_else(|_| {
                println!("Couldn't write to database file");
                exit(1);
            });

            exit(0)      
        }
    }
    
    println!("Name is not in the database");
    exit(1);
}

fn search_database(database_path: &PathBuf) -> !{
    let lock = FileLock::lock(database_path, true, 
        FileOptions::new()
            .read(true)
            .write(true)
    ).unwrap_or_else(|_| {
        println!("Couldn't open database file1");
        exit(1);
    });

    let database: Vec<Entry> = serde_json::from_reader(&lock.file).unwrap_or_else(|_| {
        println!("Couldn't read database file");
        exit(1);
    });

    enable_raw_mode().unwrap_or_else(|_| {
        println!("Couldn't create tui");
        exit(1);
    });
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap_or_else(|_| {
        println!("Couldn't create tui");
        exit(1);
    });
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap_or_else(|_| {
        println!("Couldn't create tui");
        exit(1);
    });

    let res = search_loop(&mut terminal, &database);

    disable_raw_mode().unwrap_or_else(|_| {
        println!("Couldn't destroy tui");
        exit(1);
    });
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).unwrap_or_else(|_| {
        println!("Couldn't destroy tui");
        exit(1);
    });
    terminal.show_cursor().unwrap_or_else(|_| {
        println!("Couldn't destroy tui");
        exit(1);
    });

    if let Some(command) = res.unwrap_or_else(|_|{
        println!("Couldn't create tui");
        exit(1);
    }) {
        run(&command);
    } 

    exit(0);
}

fn search_loop<'a, T: Backend>(terminal: &mut Terminal<T>, database: &'a Vec<Entry>) -> io::Result<Option<String>> {
    const TIMEOUT: Duration = Duration::from_millis(250);

    let header_style = Style::default().add_modifier(Modifier::UNDERLINED);
    let create_item = |entry: &&'a Entry| -> ListItem<'a> {
        let lines = vec![
            Spans::from(Span::styled(
                &entry.name,
                header_style
            )),
            Spans::from(entry.description.clone())
        ];
        ListItem::new(lines).style(Style::default())
    };

    let matches = |text: &str, entry: &Entry| -> bool {
        let words: Vec<&str> = text.split(" ").collect();
        words.iter().all(|word| entry.tags.iter().map(|s| s.as_str()).chain(entry.name.split(" ")).any(|tag| tag.starts_with(word)))
    };

    let mut text = String::new();
    let mut entries: Vec<&Entry> = database.iter().collect();
    let mut list: Vec<ListItem> = entries.iter().map(create_item).collect();
    let mut list_state = StatefulList::new();
    list_state.first(list.len());

    loop {

        let text_len = text.len();

        if crossterm::event::poll(TIMEOUT)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(None),
                    KeyCode::Char(c) => text.push(c),
                    KeyCode::Backspace => {text.pop();},
                    KeyCode::Up => list_state.previous(list.len()),
                    KeyCode::Down => list_state.next(list.len()),
                    KeyCode::Enter => if let Some(i) = list_state.selected() {
                        if let Some(entry) = entries.get(i) {
                            return Ok(Some(entry.command.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }

        if text_len > text.len() {
            entries.clear();
            for entry in database {
                if matches(&text, entry) {
                    entries.push(entry);
                }
            }
        
            list = entries.iter().map(create_item).collect();

            list_state.first(list.len());
        } else if text_len < text.len() {
            let mut i = 0;
            while i < entries.len() {
                if matches(&text, entries[i]) {
                    i += 1;
                }  else {
                    entries.remove(i);
                    list.remove(i);
                }
             
            }

            list_state.first(list.len());
        }
 
        terminal.draw(|f| ui(f, &text, &list, &mut list_state))?;
    
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, text: &str, entries: &[ListItem], states: &mut StatefulList) {
   let chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(2),
                ].as_ref()
        )
        .split(frame.size());



    let search = Paragraph::new(text)
        .block(Block::default().borders(Borders::BOTTOM | Borders::TOP));
    frame.render_widget(search, chunks[0]);
    frame.set_cursor(chunks[0].x + text.len() as u16, chunks[0].y + 1);

    let items = List::new(entries)
        .highlight_symbol(">");
    
    frame.render_stateful_widget(items, chunks[1], &mut states.state);

    let tool_tip = Paragraph::new("ENTER: EXEC, UP DOWN: NAV, ESC: QUIT")
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Right);
    frame.render_widget(tool_tip, chunks[2]);

}

fn run(command: &str) -> ! {
    Command::new("bash")
    .args(["-c", &command])
    .spawn()
    .unwrap_or_else(|_| {
        println!("Failed to spawn command");
        exit(1)
    })
    .wait()
    .map(|status| exit(status.code().unwrap_or(0)))
    .unwrap_or_else(|_| {
        println!("Failed to spawn command");
        exit(1)
    })
}

fn main() {
    let args = Args::parse();

    let config_path = args.config
    .map(|path| PathBuf::from(path))
    .unwrap_or_else(||
        dirs::config_dir()
        .map(|path| {
            let path = path
                .join("float-launcher");

            if !path.exists() {
                fs::create_dir(&path).unwrap_or_else(|_| {
                    println!("Couldn't create config directory");
                    exit(1)
                });
            }

            path
        })
        .unwrap_or_else(|| {
            println!("Couldn't find config path");
            exit(1)
        }));
    
    let database_path = {
        let path = config_path.join("database.json");
        if !path.exists() {
            let mut file = fs::File::create(&path).unwrap_or_else(|_| {
                println!("Couldn't create database file");
                exit(1);
            });
            file.write_all("[]".as_bytes()).unwrap_or_else(|_| {
                println!("Couldn't create database file");
                exit(1);
            });
        }
        path
    };

    match args.command {
        Some(command) => match command {
            Commands::Add { name, description, command, tags } => add_to_database(&database_path, name, description,command, tags),
            Commands::Remove { name } => remove_from_database(&database_path, &name),
            Commands::Launch => search_database(&database_path),
        },
        None => search_database(&database_path),
    }
}
