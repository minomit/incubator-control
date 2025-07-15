use chrono::{Duration, NaiveDate, Utc};
use eframe::{egui, App, Frame};
use egui::{CentralPanel, Color32, Context, RichText, Stroke, TextEdit, TopBottomPanel};
use rusqlite::{Connection, Result, ToSql};
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "incubator_sessions.db";
const APP_NAME: &str = "gestore_incubatrice_gui";

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    Italian,
    English,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Italian => write!(f, "Italiano"),
            Language::English => write!(f, "English"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self { language: Language::Italian }
    }
}

pub struct Localization {
    app_title: String,
    create_session_window_title: String,
    info_window_title: String,
    session_name_label: String,
    add_egg_batches_label: String,
    no_active_sessions_label: String,
    started_on_label: String,
    hatch_on_label: String,
    status_label: String,
    day_label: String,
    batches_in_session_label: String,
    new_session_button: String,
    add_another_batch_button: String,
    create_and_start_button: String,
    cancel_button: String,
    delete_button: String,
    info_button: String,
    preferences_button: String,
    close_button: String,
    description_hint: String,
    version_label: String,
    license_label: String,
    source_code_link: String,
    author_label: String,
}

impl Localization {
    fn new(lang: Language) -> Self {
        match lang {
            Language::Italian => Self {
                app_title: "Gestore Incubate Miste".to_string(),
                create_session_window_title: "Crea Nuova Incubata Mista".to_string(),
                info_window_title: "Informazioni".to_string(),
                session_name_label: "Nome Incubata:".to_string(),
                add_egg_batches_label: "Aggiungi Lotti di Uova:".to_string(),
                no_active_sessions_label: "Nessuna incubata attiva. Clicca su 'Nuova Incubata' per iniziare.".to_string(),
                started_on_label: "Iniziata il".to_string(),
                hatch_on_label: "Schiusa prevista".to_string(),
                status_label: "Stato".to_string(),
                day_label: "Giorno".to_string(),
                batches_in_session_label: "Lotti in questa incubata:".to_string(),
                new_session_button: "🐣 Nuova Incubata".to_string(),
                add_another_batch_button: "+ Aggiungi un altro lotto".to_string(),
                create_and_start_button: "Crea e Avvia Incubata".to_string(),
                cancel_button: "Annulla".to_string(),
                delete_button: "🗑 Elimina".to_string(),
                info_button: "Info".to_string(),
                preferences_button: "Preferenze".to_string(),
                close_button: "Chiudi".to_string(),
                description_hint: "Descrizione (es. Marans)".to_string(),
                version_label: "Versione".to_string(),
                license_label: "Licenza".to_string(),
                source_code_link: "Visita il codice sorgente su GitHub".to_string(),
                author_label: "Autore".to_string(),
            },
            Language::English => Self {
                app_title: "Mixed Batch Incubator".to_string(),
                create_session_window_title: "Create New Mixed Batch".to_string(),
                info_window_title: "About".to_string(),
                session_name_label: "Batch Name:".to_string(),
                add_egg_batches_label: "Add Egg Batches:".to_string(),
                no_active_sessions_label: "No active sessions. Click 'New Batch' to start.".to_string(),
                started_on_label: "Started on".to_string(),
                hatch_on_label: "Expected hatch".to_string(),
                status_label: "Status".to_string(),
                day_label: "Day".to_string(),
                batches_in_session_label: "Batches in this session:".to_string(),
                new_session_button: "🐣 New Batch".to_string(),
                add_another_batch_button: "+ Add another batch".to_string(),
                create_and_start_button: "Create and Start Batch".to_string(),
                cancel_button: "Cancel".to_string(),
                delete_button: "🗑 Delete".to_string(),
                info_button: "About".to_string(),
                preferences_button: "Preferences".to_string(),
                close_button: "Close".to_string(),
                description_hint: "Description (e.g., Marans)".to_string(),
                version_label: "Version".to_string(),
                license_label: "License".to_string(),
                source_code_link: "Visit source code on GitHub".to_string(),
                author_label: "Author".to_string(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Species { Gallina, Anatra, Quaglia, Oca }
impl Species {
    fn incubation_days(&self) -> i64 { match self { Self::Gallina => 21, Self::Anatra => 28, Self::Quaglia => 18, Self::Oca => 30 } }
}
impl std::fmt::Display for Species {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Batch { species: Species, description: String, egg_count: u32 }
#[derive(Clone)]
pub struct IncubationSession { id: i64, name: String, start_date: NaiveDate, batches: Vec<Batch> }
impl IncubationSession {
    fn max_incubation_days(&self) -> i64 { self.batches.iter().map(|b| b.species.incubation_days()).max().unwrap_or(0) }
    fn final_hatch_date(&self) -> NaiveDate { self.start_date + Duration::days(self.max_incubation_days()) }
    fn current_session_day(&self) -> i64 { (Utc::now().date_naive() - self.start_date).num_days() + 1 }
}

pub struct IncubatorApp {
    sessions: Vec<IncubationSession>,
    show_new_session_window: bool,
    show_about_window: bool,
    new_session_name: String,
    new_session_batches: Vec<Batch>,
    settings: Settings,
    localization: Localization,
}

impl IncubatorApp {
    fn new() -> Self {
        let conn = open_db_connection();
        init_db(&conn).expect("Creazione DB fallita");
        let settings: Settings = confy::load(APP_NAME, None).unwrap_or_default();
        let localization = Localization::new(settings.language);
        Self {
            sessions: load_sessions(&conn).expect("Caricamento sessioni fallito"),
            show_new_session_window: false,
            show_about_window: false,
            new_session_name: String::new(),
            new_session_batches: vec![],
            settings,
            localization,
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

    fn change_language(&mut self, lang: Language) {
        self.settings.language = lang;
        self.localization = Localization::new(lang);
        confy::store(APP_NAME, None, &self.settings).expect("Impossibile salvare le impostazioni");
    }
}

impl App for IncubatorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.show_new_session_window {
            egui::Window::new(&self.localization.create_session_window_title)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.localization.session_name_label);
                    ui.text_edit_singleline(&mut self.new_session_name);
                    ui.separator();
                    ui.label(&self.localization.add_egg_batches_label);

                    if self.new_session_batches.is_empty() {
                        self.new_session_batches.push(Batch {
                            species: Species::Gallina,
                            description: String::new(),
                            egg_count: 1,
                        });
                    }

                    let mut batch_to_remove = None;
                    for (i, batch) in self.new_session_batches.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_label(format!("Specie {}", i + 1))
                                .selected_text(format!("{}", batch.species))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut batch.species, Species::Gallina, "Gallina");
                                    ui.selectable_value(&mut batch.species, Species::Anatra, "Anatra");
                                    ui.selectable_value(&mut batch.species, Species::Quaglia, "Quaglia");
                                    ui.selectable_value(&mut batch.species, Species::Oca, "Oca");
                                });

                            ui.add(egui::DragValue::new(&mut batch.egg_count).clamp_range(1..=100).prefix("Uova: "));
                            let text_edit_widget = TextEdit::singleline(&mut batch.description)
                                .hint_text(&self.localization.description_hint);
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
                    if ui.button(&self.localization.add_another_batch_button).clicked() {
                        self.new_session_batches.push(Batch {
                            species: Species::Gallina,
                            description: String::new(),
                            egg_count: 1,
                        });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(&self.localization.create_and_start_button).clicked() {
                            self.add_session();
                        }
                        if ui.button(&self.localization.cancel_button).clicked() {
                            self.show_new_session_window = false;
                        }
                    });
                });
        }

        if self.show_about_window {
            egui::Window::new(&self.localization.info_window_title)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("{}: 1.0.0", &self.localization.version_label));
                    ui.label(format!("{}: {}", &self.localization.license_label, env!("CARGO_PKG_LICENSE")));
                    ui.label(format!("{}: minomitrugno", &self.localization.author_label));
                    ui.hyperlink_to(&self.localization.source_code_link, "https://github.com/minomitrugno/incubator-control");
                    if ui.button(&self.localization.close_button).clicked() {
                        self.show_about_window = false;
                    }
                });
        }

        let mut selected_language: Option<Language> = None;
        TopBottomPanel::bottom("footer")
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(&self.localization.info_button).clicked() {
                        self.show_about_window = true;
                    }
                    ui.menu_button(&self.localization.preferences_button, |ui| {
                        if ui.button("Italiano").clicked() {
                            selected_language = Some(Language::Italian);
                        }
                        if ui.button("English").clicked() {
                            selected_language = Some(Language::English);
                        }
                    });
                });
            });
        if let Some(lang) = selected_language {
            self.change_language(lang);
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&self.localization.app_title);
                if ui.button(&self.localization.new_session_button).clicked() {
                    self.show_new_session_window = true;
                }
            });
            ui.separator();

            if self.sessions.is_empty() {
                ui.label(&self.localization.no_active_sessions_label);
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
                                if ui.button(&self.localization.delete_button).clicked() {
                                    session_to_remove = Some(session.id);
                                }
                            });
                        });
                        ui.label(format!(
                            "{}: {}. {}: {}",
                            &self.localization.started_on_label,
                            session.start_date.format("%d/%m/%Y"),
                            &self.localization.hatch_on_label,
                            session.final_hatch_date().format("%d/%m/%Y")
                        ));

                        ui.add_space(5.0);
                        ui.label(format!(
                            "{}: {}",
                            &self.localization.status_label,
                            current_day.max(0)
                        ));
                        ui.add(egui::ProgressBar::new(progress.clamp(0.0, 1.0)).show_percentage());
                        ui.add_space(10.0);

                        ui.label(RichText::new(&self.localization.batches_in_session_label).strong());

                        for batch in &session.batches {
                            let day_to_add = max_days - batch.species.incubation_days() + 1;
                            let text: RichText;

                            if current_day == day_to_add {
                                text = RichText::new(format!(
                                    "➡️ {}: {} ({})",
                                    &self.localization.day_label,
                                    batch.species,
                                    batch.description
                                ))
                                .color(Color32::GREEN)
                                .strong()
                                .size(16.0);
                            } else if current_day < day_to_add {
                                text = RichText::new(format!(
                                    "⏳ {} {} ({}) {} {}",
                                    &self.localization.add_egg_batches_label,
                                    batch.species,
                                    batch.description,
                                    &self.localization.day_label,
                                    day_to_add
                                ))
                                .color(Color32::GRAY);
                            } else {
                                text = RichText::new(format!(
                                    "✅ {} {} ({})",
                                    batch.species, batch.description, batch.egg_count
                                ))
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

pub fn start() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|_cc| Box::new(IncubatorApp::new())),
    ).expect("Impossibile avviare eframe");
}