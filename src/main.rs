mod settings;
mod music_player;
mod db;

slint::include_modules!();

use std::{rc::Rc, sync::{Arc, Mutex}, time::SystemTime};
use anyhow::Result;
use chrono::Utc;
use single_instance::SingleInstance;
use slint::{ComponentHandle, Model, ModelRc, PlatformError, Timer, TimerMode, VecModel};

use settings::JsonSettings;
use music_player::MusicPlayer;
use db::{Database, Task as TaskStruct};

pub const PROGRESS_BYTES: &str = include_str!("../assets/Circle.svg");


impl AppWindow {
    fn set_settings(&self, settings: &JsonSettings) {
        self.global::<Settings>()
            .set_always_on_top(settings.always_on_top);

        self.global::<Settings>()
            .set_auto_start_focus_timer(settings.auto_start_focus_timer);

        self.global::<Settings>()
            .set_auto_start_break_timer(settings.auto_start_break_timer);
        
        self.global::<Settings>()
            .set_minimize_to_tray(settings.minimize_to_tray);

        self.global::<Settings>()
            .set_minimize_to_tray_on_close(settings.minimize_to_tray_on_close);

        self.global::<Settings>()
            .set_notifications(settings.notifications);

        self.global::<Settings>()
            .set_theme((&settings.theme).into());

        self.global::<Settings>()
            .set_song((&settings.song).into());
        
        self.global::<Settings>()
            .set_focus(settings.focus);
    
        self.global::<Settings>()
            .set_short_break(settings.short_break);

        self.global::<Settings>()
            .set_long_break(settings.long_break);

        self.global::<Settings>()
            .set_rounds(settings.rounds);

        self.global::<Settings>()
            .set_ambient_volume(settings.ambient_volume);

        self.global::<Settings>()
            .set_alerts_volume(settings.alerts_volume)

    }

    fn save_settings(&self) {
        settings::save_settings(
            JsonSettings {
                always_on_top: self.global::<Settings>().get_always_on_top(),
                auto_start_focus_timer: self.global::<Settings>().get_auto_start_focus_timer(),
                auto_start_break_timer: self.global::<Settings>().get_auto_start_break_timer(),
                minimize_to_tray: self.global::<Settings>().get_minimize_to_tray(),
                minimize_to_tray_on_close: self.global::<Settings>().get_minimize_to_tray_on_close(),
                notifications: self.global::<Settings>().get_notifications(),
                theme: self.global::<Settings>().get_theme().to_string(),
                song: self.global::<Settings>().get_song().to_string(),
                focus: self.global::<Settings>().get_focus(),
                short_break: self.global::<Settings>().get_short_break(),
                long_break: self.global::<Settings>().get_long_break(),
                rounds: self.global::<Settings>().get_rounds(),
                ambient_volume: self.global::<Settings>().get_ambient_volume(),
                alerts_volume: self.global::<Settings>().get_alerts_volume(),
            }
        )
    }
}

struct Tranquilo {
    pub window: AppWindow,
    settings: JsonSettings,
    music_player: Arc<Mutex<MusicPlayer>>,
    tasks: Arc<Mutex<Database>>
}

impl Tranquilo {
    fn new() -> Self {
        let settings: JsonSettings = settings::load_settings();
        let themes: Vec<JsonTheme> = settings::load_themes();
        let songs: Vec<String> = settings::load_songs();
        
        let window = AppWindow::new().unwrap();
        window.set_settings(&settings);

        let themes_model: Rc<VecModel<JsonTheme>> = Rc::new(VecModel::from(themes));
        window.global::<ThemeCallbacks>().set_themes(ModelRc::from(themes_model.clone()));

        let songs_model: Rc<VecModel<slint::SharedString>> = Rc::new(VecModel::from(
            songs.into_iter().map(|s| slint::SharedString::from(s)).collect::<Vec<_>>()
        ));
        window.global::<SongCallbacks>().set_songs(ModelRc::from(songs_model.clone()));

        let music_player = Arc::new(Mutex::new(MusicPlayer::new()));

        let db = Arc::new(Mutex::new(Database::new("assets/tasks.db")));
        db.lock().unwrap().init_db();

        let tasks_model = Rc::new(VecModel::from(
            db.lock().unwrap().fetch_tasks().unwrap()
        ));
        window.global::<Tasks>().set_tasks(ModelRc::from(tasks_model.clone()));

        Self {
            window,
            settings,
            music_player,
            tasks: db
        }
    }
    
    fn update_progress_svg(remaining_period: f32) -> slint::Image {
        slint::Image::load_from_svg_data(
            PROGRESS_BYTES.replace(
                "stroke-dasharray=\"100, 100\"",
                &format!("stroke-dasharray=\"{}, 100\"", remaining_period)
            )
            .as_bytes()
        )
        .unwrap()
    }

    fn run(&self) -> Result<(), PlatformError> {
        let theme_name = &self.settings.theme;
        let themes = self.window.global::<ThemeCallbacks>().get_themes();
        let (index, current_theme) = themes
            .iter()
            .enumerate()
            .find(|(_, theme)| theme.name == theme_name)
            .unwrap();
        self.window.global::<ThemeCallbacks>().invoke_theme_changed(index as i32, current_theme.clone());

        let song_name = &self.settings.song;
        let songs = self.window.global::<SongCallbacks>().get_songs();
        let (_index, current_song) = songs
            .iter()
            .enumerate()
            .find(|(_, song)| song == song_name)
            .unwrap();
        self.window.global::<SongCallbacks>().invoke_song_changed(current_song.clone());

        self.window.run()
    }
}

fn main() -> Result<()> {
    let instance = SingleInstance::new("io.risuleia.tranquilo").unwrap();
    if !instance.is_single() {
        return Err(anyhow::anyhow!("One instance of Tranquilo is already running."));
    }

    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap())).unwrap();

    let tranquilo = Tranquilo::new();
    let music_handle = Arc::clone(&tranquilo.music_player);

    let set_bool_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<Settings>()
        .on_bool_changed(move |setting_type, value| {
            let setting_handle = set_bool_handle.upgrade().unwrap();

            match setting_type {
                BoolSettingTypes::AlwaysOnTop => {
                    setting_handle.global::<Settings>().set_always_on_top(!value)
                }
                BoolSettingTypes::AutoStartFocusTimer => {
                    setting_handle.global::<Settings>().set_auto_start_focus_timer(!value)
                }
                BoolSettingTypes::AutoStartBreakTimer => {
                    setting_handle.global::<Settings>().set_auto_start_break_timer(!value)
                }
                BoolSettingTypes::MinimizeToTray => {
                    setting_handle.global::<Settings>().set_minimize_to_tray(!value)
                }
                BoolSettingTypes::MinimizeToTrayOnClose => {
                    setting_handle.global::<Settings>().set_minimize_to_tray_on_close(!value)
                }
                BoolSettingTypes::Notifications => {
                    setting_handle.global::<Settings>().set_notifications(!value)
                }
            }
            setting_handle.save_settings();
        });

    let set_int_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<Settings>()
        .on_int_changed(move |setting_type, value| {
            let setting_handle = set_int_handle.upgrade().unwrap();
            let music_handle = &mut *music_handle.lock().unwrap();
            
            match setting_type {
                IntSettingTypes::Focus => {
                    setting_handle.global::<Settings>().set_focus(value);
                }
                IntSettingTypes::ShortBreak => {
                    setting_handle.global::<Settings>().set_short_break(value)
                }
                IntSettingTypes::LongBreak => {
                    setting_handle.global::<Settings>().set_long_break(value)
                }
                IntSettingTypes::Rounds => {
                    setting_handle.global::<Settings>().set_rounds(value)
                }
                IntSettingTypes::AmbientVolume => {
                    setting_handle.global::<Settings>().set_ambient_volume(value);
                    music_handle.set_volume(value as f32 / 100.0);
                }
                IntSettingTypes::AlertsVolume => {
                    setting_handle.global::<Settings>().set_alerts_volume(value)
                }
            }
            setting_handle.save_settings();
        });

    let close_handle = tranquilo.window.as_weak();
    tranquilo.window.on_close_window(move || {
        let close_handle = close_handle.upgrade().unwrap();
        close_handle.hide().unwrap();
        });

    let minimize_handle = tranquilo.window.as_weak();
    tranquilo.window.on_minimize_window(move || {
        let minimize_handle = minimize_handle.upgrade().unwrap();
        i_slint_backend_winit::WinitWindowAccessor::with_winit_window(
            minimize_handle.window(),
            |window| window.set_minimized(true)
        );
        });

    let move_hande = tranquilo.window.as_weak();
    tranquilo.window.on_move_window(move || {
        let move_handle = move_hande.upgrade().unwrap();
        i_slint_backend_winit::WinitWindowAccessor::with_winit_window(
            move_handle.window(),
            |window| window.drag_window()
        );
        });
    
    let theme_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<ThemeCallbacks>()
        .on_theme_changed(move |index, theme| {
            let theme_handle = theme_handle.upgrade().unwrap();
            theme_handle.global::<Settings>().set_theme(theme.name);
            theme_handle.save_settings();

            theme_handle.global::<Theme>().set_theme_index(index);
            theme_handle.global::<Theme>().set_focus_round(theme.focus_round);
            theme_handle.global::<Theme>().set_short_break(theme.short_break);
            theme_handle.global::<Theme>().set_long_break(theme.long_break);
            theme_handle.global::<Theme>().set_background(theme.background);
            theme_handle.global::<Theme>().set_background_light(theme.background_light);
            theme_handle.global::<Theme>().set_background_lightest(theme.background_lightest);
            theme_handle.global::<Theme>().set_foreground(theme.foreground);
            theme_handle.global::<Theme>().set_foreground_darker(theme.foreground_darker);
            theme_handle.global::<Theme>().set_foreground_darkest(theme.foreground_darkest);
            theme_handle.global::<Theme>().set_accent(theme.accent);
            theme_handle.global::<Theme>().set_nav(theme.nav);
            theme_handle.global::<Theme>().set_text_clr_primary(theme.text_clr_primary);
            theme_handle.global::<Theme>().set_text_clr_secondary(theme.text_clr_secondary);
        });


    let music_player = Arc::clone(&tranquilo.music_player);
    let timer = Arc::new(Timer::default());
    let timer_handle = Arc::new(tranquilo.window.as_weak());
    tranquilo.window.on_action_timer(move |action| {
        let timer_start = timer_handle.clone();
        let timer_handle = timer_handle.upgrade().unwrap();
        let music_handle = &mut *music_player.lock().unwrap();

        match action {
            TimerAction::Start => {
                
                timer_handle.set_active(true);
                timer.start(
                    TimerMode::Repeated,
                    std::time::Duration::from_millis(50),
                    move || {
                        let timer_start = timer_start.upgrade().unwrap();
                        timer_start.invoke_tick(50);
                        
                        let remaining_time = timer_start.get_remaining_time() as f32;
                        let target_time = timer_start.get_target_time() as f32;
                        
                        let remaining_period = remaining_time / target_time * 100.0;

                        timer_start.set_circle_progress(Tranquilo::update_progress_svg(remaining_period));
                    }
                );
                
                let song_name = timer_handle.global::<Settings>().get_song().to_string();
                let volume = timer_handle.get_ambient_volume() as f32 / 100.0;
                music_handle.play(song_name);
                music_handle.set_volume(volume);
            }
            TimerAction::Stop => {
                timer_handle.set_active(false);
                timer.stop();
                music_handle.pause();
            }
            TimerAction::Reset => {
                timer_handle.set_active(false);
                timer_handle.set_remaining_time(<i32 as Into<i64>>::into(timer_handle.global::<Settings>().get_focus()) * 60000);
                timer_handle.set_active_round(1);
                timer_handle.set_active_timer(TimerType::Focus);
                timer.stop();
                
                timer_handle.set_circle_progress(Tranquilo::update_progress_svg(100.0));
                music_handle.stop();
            }
            TimerAction::Skip => {
                timer_handle.invoke_change_timer();
            }
        }
        });

    let timer_change_handle = tranquilo.window.as_weak();
    tranquilo.window.on_change_timer(move || {
        let timer_change_handle = timer_change_handle.upgrade().unwrap();

        match timer_change_handle.get_active_timer() {
            TimerType::Focus => {
                if timer_change_handle.get_active_round() == timer_change_handle.get_timer_config().rounds {
                    let long_break = timer_change_handle.get_timer_config().long_break;

                    timer_change_handle.set_active_round(1);
                    timer_change_handle.set_active_timer(TimerType::LongBreak);
                    timer_change_handle.set_target_time(long_break);
                    timer_change_handle.set_remaining_time(long_break);
                } else {
                    let short_break = timer_change_handle.get_timer_config().short_break;

                    timer_change_handle.set_active_timer(TimerType::ShortBreak);
                    timer_change_handle.set_target_time(short_break);
                    timer_change_handle.set_remaining_time(short_break);
                }
            }
            TimerType::ShortBreak => {
                let focus_time = timer_change_handle.get_timer_config().focus;

                timer_change_handle.set_active_round(timer_change_handle.get_active_round() + 1);
                timer_change_handle.set_active_timer(TimerType::Focus);
                timer_change_handle.set_target_time(focus_time);
                timer_change_handle.set_remaining_time(focus_time);
            }
            TimerType::LongBreak => {
                let focus_time = timer_change_handle.get_timer_config().focus;

                timer_change_handle.set_active_round(1);
                timer_change_handle.set_active_timer(TimerType::Focus);
                timer_change_handle.set_target_time(focus_time);
                timer_change_handle.set_remaining_time(focus_time);
            }
        }
        });
    
    let music_change_handle = Arc::clone(&tranquilo.music_player);
    let song_change_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<SongCallbacks>()
        .on_song_changed(move |song| {
            let cloned_song = song.clone();
            let song_change_handle = song_change_handle.upgrade().unwrap();
            let music_change_handle = &mut *music_change_handle.lock().unwrap();

            song_change_handle.global::<Settings>().set_song(song);
            song_change_handle.save_settings();

            if song_change_handle.get_active() {
                music_change_handle.stop();
                music_change_handle.play(cloned_song.to_string())
            }
        });
    
        
    let tasks_status_handle = Arc::clone(&tranquilo.tasks);
    let task_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<Tasks>()
        .on_set_status(move |id, status| {
            let task_status_handle = &mut *tasks_status_handle.lock().unwrap();
            let task_handle = task_handle.upgrade().unwrap();

            let _ = task_status_handle.update_task_state(id.to_string(), status);

            let tasks = ModelRc::new(VecModel::from(task_status_handle.fetch_tasks().unwrap()));
            task_handle.global::<Tasks>().set_tasks(ModelRc::from(tasks));
        });
    
    
    let tasks_add_handle = Arc::clone(&tranquilo.tasks);
    let task_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<Tasks>()
        .on_add_task(move |text| {

            if text.to_string() == "" {
                return;
            }

            let task_add_handle = &mut *tasks_add_handle.lock().unwrap();
            let task_handle = task_handle.upgrade().unwrap();

            let task = &TaskStruct {
                id: std::time::SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis()
                    .to_string(),
                text: text.into(),
                status: TaskStatus::Pending,
                timestamp: Utc::now().to_string()
            };
            let _ = task_add_handle.insert_task(task);

            let tasks = ModelRc::new(VecModel::from(task_add_handle.fetch_tasks().unwrap()));
            task_handle.global::<Tasks>().set_tasks(ModelRc::from(tasks));
        });

    let tasks_remove_handle = Arc::clone(&tranquilo.tasks);
    let task_handle = tranquilo.window.as_weak();
    tranquilo.window.global::<Tasks>()
        .on_delete_task(move |id| {
            let task_remove_handle = &mut *tasks_remove_handle.lock().unwrap();
            let task_handle = task_handle.upgrade().unwrap();

            let _ = task_remove_handle.remove_task(id.to_string());

            let tasks = ModelRc::new(VecModel::from(task_remove_handle.fetch_tasks().unwrap()));
            task_handle.global::<Tasks>().set_tasks(ModelRc::from(tasks));
        });
    
    tranquilo.run()?;
    Ok(())

}