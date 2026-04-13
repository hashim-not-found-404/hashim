mod elements;
use client_reqwest;
use eframe::egui;
use getrandom;
use my_core::{
    front_end_model_view::{self, Signal},
    traits,
};
use std::sync::{LazyLock, RwLock, RwLockWriteGuard};
use tokio;

pub struct Nnnn;
impl traits::TransactionNumber for Nnnn {
    fn generate() -> u64 {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).unwrap();
        u64::from_ne_bytes(buf)
    }
}

type GG = front_end_model_view::State<
    client_reqwest::MyClient,
    Nnnn,
    MySignal<String>,
    MySignal<bool>,
    MySignal<String>,
>;

pub static STATE: LazyLock<GG> = LazyLock::new(|| GG::new(client_reqwest::MyClient::default()));

#[derive(Default)]
pub struct MySignal<T: Default>(RwLock<T>);

impl<T: Clone + Default> MySignal<T> {
    fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }
}

impl<T: Clone + Default> Signal for MySignal<T> {
    type T = T;
    fn read(&self) -> Self::T {
        self.0.read().unwrap().clone()
    }
    fn set(&self, v: Self::T) {
        *self.0.write().unwrap() = v
    }
}

#[derive(Clone, PartialEq, Default)]
enum Page {
    #[default]
    SignIn,
    SignUp,
    Home,
}

static CURRENT_PAGE: LazyLock<MySignal<Page>> = LazyLock::new(|| MySignal::default());

const APP_NAME: &str = "accounting app";

#[tokio::main]
async fn main() {
    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(MyApp {}))),
    )
    .unwrap();
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let page = CURRENT_PAGE.read();

                if STATE.is_signed_in.read() {
                    CURRENT_PAGE.set(Page::Home);
                } else if page != Page::SignIn && page != Page::SignUp {
                    CURRENT_PAGE.set(Page::SignIn);
                }

                match page {
                    Page::SignIn => sign_in_page(ui),
                    Page::SignUp => sign_up_page(ui),
                    Page::Home => home_page(ui),
                }
                error_box(ui);
            });
    }
}

fn error_box(ui: &mut egui::Ui) {
    if STATE.external_errors.read() != String::new() {
        ui.horizontal(|ui| {
            if ui.button("X").clicked() {
                STATE.external_errors.set(String::new());
            }
            ui.label(STATE.external_errors.read());
        });
    }
}

fn sign_in_page(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        {
            static DROW_STATE: RwLock<elements::Input1> = RwLock::new(elements::Input1::new());
            DROW_STATE
                .write()
                .unwrap()
                .draw(ui, &mut STATE.user_id.write(), "user id", false);
        }
        elements::error_label(ui, STATE.sign_in_error_for_user_id.read().as_str());

        elements::input_password(ui, &mut STATE.password.write(), "password");
        elements::error_label(ui, STATE.sign_in_error_for_password.read().as_str());

        {
            static DROW_STATE: RwLock<elements::Btn1> = RwLock::new(elements::Btn1::new());
            if DROW_STATE.write().unwrap().draw(
                ui,
                "sign in",
                STATE.is_loading_sign_in_or_up.read(),
            ) {
                spawn(&ui, STATE.sign_in());
            };
        }
        {
            static DROW_STATE: RwLock<elements::Btn2> = RwLock::new(elements::Btn2::new());
            if DROW_STATE.write().unwrap().draw(ui, "sign up", false) {
                CURRENT_PAGE.set(Page::SignUp);
            };
        }
    });
}

fn sign_up_page(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        {
            static DROW_STATE: RwLock<elements::Input1> = RwLock::new(elements::Input1::new());
            DROW_STATE
                .write()
                .unwrap()
                .draw(ui, &mut STATE.user_name.write(), "user name", false);
        }
        elements::error_label(ui, STATE.sign_up_error_for_user_name.read().as_str());

        {
            static DROW_STATE: RwLock<elements::Input1> = RwLock::new(elements::Input1::new());
            DROW_STATE
                .write()
                .unwrap()
                .draw(ui, &mut STATE.user_id.write(), "user id", false);
        }
        // elements::input_1(ui, &mut STATE.user_id.write(), "user id");
        elements::error_label(ui, STATE.sign_up_error_for_user_id.read().as_str());

        elements::input_password(ui, &mut STATE.password.write(), "password");

        {
            static DROW_STATE: RwLock<elements::Btn1> = RwLock::new(elements::Btn1::new());
            if DROW_STATE.write().unwrap().draw(
                ui,
                "sign up",
                STATE.is_loading_sign_in_or_up.read(),
            ) {
                spawn(&ui, STATE.sign_up());
            };
        }
        {
            static DROW_STATE: RwLock<elements::Btn2> = RwLock::new(elements::Btn2::new());
            if DROW_STATE.write().unwrap().draw(ui, "back", false) {
                CURRENT_PAGE.set(Page::SignIn);
            };
        }
    });
}

fn home_page(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {});
}

fn spawn<F>(ui: &egui::Ui, f: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let ctx = ui.ctx().clone();
    tokio::spawn(async move {
        f.await;
        ctx.request_repaint();
    });
}
