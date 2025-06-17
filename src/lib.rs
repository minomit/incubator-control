use chrono::{Duration, NaiveDate, Utc};
use eframe::{egui, App, Frame};
use egui::{CentralPanel, Color32, Context, RichText, Stroke, TextEdit};
use rusqlite::{Connection, Result, ToSql};
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "incubator_sessions.db";

// --- Strutture Dati e Logica di Base ---

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Species {
    Gallina, Anatra, Quaglia, Oca,
}

impl Species {
    fn incubation_days(&self) -> i64 {
        match self {
            Self::Gallina => 21, Self::Anatra => 28, Self::Quaglia => 18, Self::Oca => 30,
        }
    }
}

impl std::fmt::Display for Species {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Batch {
    species: Species,
    description: String,
}

#[derive(Clone)]
pub struct IncubationSession {
    id: i64,
    name: String,
    start_date: NaiveDate,
    batches: Vec<Batch>,
}

impl IncubationSession {
    fn max_incubation_days(&self) -> i64 {
        self.batches.iter().map(|b| b.species.incubation_days()).max().unwrap_or(0)
    }
    
    fn final_hatch_date(&self) -> NaiveDate {
        self.start_date + Duration::days(self.max_incubation_days())
    }

    fn current_session_day(&self) -> i64 {
        let today = Utc::now().date_naive();
        (today - self.start_date).num_days() + 1
    }
}

// --- Struttura Principale dell'App ---

pub struct IncubatorApp {
    sessions: Vec<IncubationSession>,
    show_new_session_window: bool,
    new_session_name: String,
    new_session_batches: Vec<Batch>,
}

impl IncubatorApp {
    fn new() -> Self {
        let conn = open_db_connection();
        init_db(&conn).expect("Creazione DB fallita");
        
        Self {
            sessions: load_sessions(&conn).expect("Caricamento sessioni fallito"),
            show_new_session_window: false,
            new_session_name: String::new(),
            new_session_batches: vec![],
        }
    }

    fn add_session(&mut self) {
        if !self.new_session_name.is_empty() && !self.new_session_batches.is_empty() {
            let session = IncubationSession {
                id: 0,
                name: self.new_session_name.clone(),
                start_date: Utc::now().date_naive(),
                batches: self.new_session_batches.clone(),
            };
            let conn = open_db_connection();
            if add_session_to_db(&conn, &session).is_ok() {
                self.sessions = load_sessions(&conn).unwrap();
            }
            self.show_new_session_window = false;
            self.new_session_name.clear();
            self.new_session_batches.clear();
        }
    }
}

impl App for IncubatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.show_new_session_window {
            egui::Window::new("Crea Nuova Incubata Mista")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Nome Incubata:");
                    ui.text_edit_singleline(&mut self.new_session_name);
                    ui.separator();
                    ui.label("Aggiungi Lotti di Uova:");
                    
                    if self.new_session_batches.is_empty() {
                       self.new_session_batches.push(Batch {
                           species: Species::Gallina,
                           description: String::new(),
                       });
                    }

                    let mut batch_to_remove = None;
                    for (i, batch) in self.new_session_batches.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                             egui::ComboBox::from_label(format!("Specie {}", i+1))
                                .selected_text(format!("{}", batch.species))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut batch.species, Species::Gallina, "Gallina");
                                    ui.selectable_value(&mut batch.species, Species::Anatra, "Anatra");
                                    ui.selectable_value(&mut batch.species, Species::Quaglia, "Quaglia");
                                    ui.selectable_value(&mut batch.species, Species::Oca, "Oca");
                                });
                            
                            let text_edit_widget = TextEdit::singleline(&mut batch.description)
                                .hint_text("Descrizione (es. Marans)");
                            ui.add(text_edit_widget);

                            if ui.button("🗑").clicked() {
                                batch_to_remove = Some(i);
                            }
                        });
                    }
                    if let Some(i) = batch_to_remove {
                        self.new_session_batches.remove(i);
                    }

                    ui.add_space(5.0);
                    if ui.button("+ Aggiungi un altro lotto").clicked() {
                        self.new_session_batches.push(Batch { species: Species::Gallina, description: String::new() });
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Crea e Avvia Incubata").clicked() {
                            self.add_session();
                        }
                        if ui.button("Annulla").clicked() {
                            self.show_new_session_window = false;
                        }
                    });
                });
        }
        
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Gestore di Incubate");
                if ui.button("🐣 Nuova Incubata").clicked() {
                    self.show_new_session_window = true;
                }
            });
            ui.separator();
            
            if self.sessions.is_empty() {
                ui.label("Nessuna incubata attiva. Clicca su 'Nuova Incubata' per iniziare.");
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut session_to_remove: Option<i64> = None;
                for session in &self.sessions {
                    let max_days = session.max_incubation_days();
                    let current_day = session.current_session_day();
                    let progress = if max_days > 0 { (current_day as f32) / (max_days as f32) } else { 0.0 };

                    let frame = egui::Frame::group(ui.style()).stroke(Stroke::new(1.0, Color32::GRAY));
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new(&session.name).size(20.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑 Elimina").clicked() {
                                    session_to_remove = Some(session.id);
                                }
                            });
                        });
                        ui.label(format!("Iniziata il: {}. Schiusa prevista: {}", 
                            session.start_date.format("%d/%m/%Y"), 
                            session.final_hatch_date().format("%d/%m/%Y")));
                        
                        ui.add_space(5.0);
                        ui.label(format!("Stato: Giorno {} di {}", current_day.max(0), max_days));
                        ui.add(egui::ProgressBar::new(progress.clamp(0.0, 1.0)).show_percentage());
                        ui.add_space(10.0);

                        ui.label(RichText::new("Lotti in questa incubata:").strong());
                        
                        for batch in &session.batches {
                            let day_to_add = max_days - batch.species.incubation_days() + 1;
                            let text: RichText;
                            
                            if current_day == day_to_add {
                                text = RichText::new(format!("➡️ OGGI: Inserisci le uova di {} ({})", batch.species, batch.description))
                                    .color(Color32::GREEN).strong().size(16.0);
                            } else if current_day < day_to_add {
                                text = RichText::new(format!("⏳ Inserisci le uova di {} ({}) al giorno {}", batch.species, batch.description, day_to_add))
                                    .color(Color32::GRAY);
                            } else {
                                text = RichText::new(format!("✅ Uova di {} ({}) inserite", batch.species, batch.description))
                                    .color(Color32::from_rgb(100, 150, 100));
                            }
                            ui.label(text);
                        }
                    });
                    ui.add_space(10.0);
                }
                
                if let Some(id) = session_to_remove {
                    let conn = open_db_connection();
                    if remove_session_from_db(&conn, id).is_ok() {
                        self.sessions.retain(|s| s.id != id);
                    }
                }
            });
        });
    }
}


fn open_db_connection() -> Connection {
    Connection::open(DB_PATH).expect("Connessione DB fallita")
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            start_date  DATE NOT NULL,
            batches     TEXT NOT NULL
        )",
        (),
    )?;
    Ok(())
}

fn add_session_to_db(conn: &Connection, session: &IncubationSession) -> Result<i64> {
    let batches_json = serde_json::to_string(&session.batches).unwrap();
    
    conn.execute(
        "INSERT INTO sessions (name, start_date, batches) VALUES (?1, ?2, ?3)",
        &[&session.name as &dyn ToSql, &session.start_date, &batches_json],
    )?;
    Ok(conn.last_insert_rowid())
}

fn remove_session_from_db(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", [id])
}

fn load_sessions(conn: &Connection) -> Result<Vec<IncubationSession>> {
    let mut stmt = conn.prepare("SELECT id, name, start_date, batches FROM sessions ORDER BY start_date DESC")?;
    let session_iter = stmt.query_map([], |row| {
        let batches_json: String = row.get(3)?;
        let batches: Vec<Batch> = serde_json::from_str(&batches_json).unwrap_or_else(|_| vec![]);
        
        Ok(IncubationSession {
            id: row.get(0)?,
            name: row.get(1)?,
            start_date: row.get(2)?,
            batches,
        })
    })?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session?);
    }
    Ok(sessions)
}

// --- Punti di Ingresso per le Piattaforme ---

/// Questa funzione viene chiamata da `main.rs` per avviare l'app su desktop.
pub fn start() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gestore Incubate Miste",
        native_options,
        Box::new(|_cc| Box::new(IncubatorApp::new())),
    ).expect("Impossibile avviare eframe");
}

/// Questo è il punto di ingresso per Android, chiamato dal sistema operativo.
#[cfg(target_os = "android")]
pub fn android_main(_cc: &eframe::CreationContext) -> Box<dyn eframe::App> {
    Box::new(IncubatorApp::new())
}