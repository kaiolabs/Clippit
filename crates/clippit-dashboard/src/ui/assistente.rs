//! Página "Assistente": o Alt+K que lê a conversa da tela e sugere a resposta.
//! Seus nomes vão para o corretor.conf; os contextos são arquivos .md em
//! ~/.config/clippit/contextos (1ª linha opcional "apps: whatsapp, slack").

use super::conf;
use super::corretor::{key_row, switch_row};
use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

struct Ctx {
    name: String,
    apps: Vec<String>,
    preview: String,
    path: std::path::PathBuf,
}

fn load_contexts() -> Vec<Ctx> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(conf::contexts_dir()) {
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("md") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let mut lines = text.lines();
            let mut apps = Vec::new();
            let mut first = lines.next().unwrap_or("").to_string();
            if first.to_lowercase().starts_with("apps:") {
                apps = first[5..]
                    .split(',')
                    .map(|a| a.trim().to_string())
                    .filter(|a| !a.is_empty())
                    .collect();
                first = lines.next().unwrap_or("").to_string();
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            out.push(Ctx {
                name,
                apps,
                preview: first.trim().chars().take(90).collect(),
                path,
            });
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

fn fill(group: &adw::PreferencesGroup, rows: &Rc<RefCell<Vec<adw::ActionRow>>>) {
    for r in rows.borrow_mut().drain(..) {
        group.remove(&r);
    }
    let contexts = load_contexts();
    if contexts.is_empty() {
        let row = adw::ActionRow::new();
        row.set_title("Nenhum contexto ainda");
        row.set_subtitle("Crie um abaixo, ou pelo próprio painel do Alt+K");
        group.add(&row);
        rows.borrow_mut().push(row);
        return;
    }
    for ctx in contexts {
        let row = adw::ActionRow::new();
        row.set_title(&ctx.name);
        let auto = if ctx.apps.is_empty() {
            "Escolha manual".to_string()
        } else {
            format!("Automático em: {}", ctx.apps.join(", "))
        };
        row.set_subtitle(&if ctx.preview.is_empty() {
            auto
        } else {
            format!("{auto}  ·  {}", ctx.preview)
        });
        let edit = gtk::Button::from_icon_name("document-edit-symbolic");
        edit.add_css_class("clip-icon");
        edit.set_valign(gtk::Align::Center);
        edit.set_tooltip_text(Some("Editar"));
        let path = ctx.path.clone();
        edit.connect_clicked(move |_| conf::open(&path));
        row.add_suffix(&edit);
        row.set_activatable_widget(Some(&edit));
        group.add(&row);
        rows.borrow_mut().push(row);
    }
}

fn learned() -> (u64, u64, u64, u64, u64, std::path::PathBuf) {
    let cmd = dirs::home_dir()
        .unwrap_or_default()
        .join(".local/bin/clippit-correct");
    let out = std::process::Command::new(cmd).arg("--learned").output();
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    // JSON pequeno e fixo: {"respostas": 3, "editadas": 1, "palavras": 2, "trocas": 4, "pasta": "..."}
    let num = |key: &str| -> u64 {
        text.split(&format!("\"{key}\": "))
            .nth(1)
            .and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    };
    let dir = dirs::data_dir()
        .unwrap_or_default()
        .join("clippit")
        .join("aprendizado");
    (
        num("respostas"),
        num("editadas"),
        num("palavras"),
        num("trocas"),
        num("pessoas"),
        dir,
    )
}

fn learning_group(window_hint: &gtk::Widget) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    group.set_title("Aprendizado");
    group.set_description(Some(
        "O Clippit aprende com você, tudo no seu computador: as respostas que você insere viram exemplo \
         para as próximas, as palavras que você ignora no corretor entram no seu dicionário e as trocas \
         que você aceita várias vezes passam a sair sozinhas.",
    ));
    group.add(&switch_row(
        "Aprender com o que eu uso",
        "Desligado: não guarda nada novo (o que já foi aprendido continua valendo)",
        "starred-symbolic",
        "aprender",
        true,
    ));

    let summary = adw::ActionRow::new();
    summary.set_title("O que já foi aprendido");
    let open = gtk::Button::with_label("Ver arquivos");
    open.add_css_class("clip-btn");
    open.set_valign(gtk::Align::Center);
    let clear = gtk::Button::with_label("Apagar tudo");
    clear.add_css_class("clip-btn");
    clear.set_valign(gtk::Align::Center);
    summary.add_suffix(&open);
    summary.add_suffix(&clear);
    group.add(&summary);

    // Treino (LoRA): junta o aprendido num arquivo; avisa quando já é suficiente.
    let train = adw::ActionRow::new();
    train.set_title("Dados para treinar o modelo");
    train.set_subtitle("Com umas 300 respostas e correções guardadas dá para fazer um ajuste fino (LoRA) do modelo com o seu jeito");
    let export = gtk::Button::with_label("Exportar");
    export.add_css_class("clip-btn");
    export.set_valign(gtk::Align::Center);
    {
        let train = train.clone();
        export.connect_clicked(move |_| {
            let cmd = dirs::home_dir()
                .unwrap_or_default()
                .join(".local/bin/clippit-correct");
            if let Ok(o) = std::process::Command::new(cmd)
                .arg("--exportar-treino")
                .output()
            {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                train.set_subtitle(&text.replace('\n', " "));
            }
        });
    }
    train.add_suffix(&export);
    group.add(&train);

    let refresh = {
        let summary = summary.clone();
        let clear = clear.clone();
        move || {
            let (resp, edit, words, fixes, ppl, _) = learned();
            summary.set_subtitle(&format!(
                "{resp} respostas guardadas ({edit} editadas por você) · {ppl} pessoas com anotações · \
                 {words} palavras no seu dicionário · {fixes} trocas aprendidas"
            ));
            clear.set_sensitive(resp + words + fixes + ppl > 0);
        }
    };
    refresh();
    open.connect_clicked(|_| {
        let (.., dir) = learned();
        let _ = std::fs::create_dir_all(&dir);
        conf::open(&dir);
    });
    {
        let refresh = refresh.clone();
        let hint = window_hint.clone();
        clear.connect_clicked(move |_| {
            let dialog = adw::MessageDialog::new(
                hint.root().and_downcast_ref::<gtk::Window>(),
                Some("Apagar tudo o que foi aprendido?"),
                Some("As respostas guardadas, as anotações sobre pessoas, o seu dicionário, as trocas aprendidas e as frases da prova serão apagados. Não dá para desfazer."),
            );
            dialog.add_response("cancel", "Cancelar");
            dialog.add_response("clear", "Apagar");
            dialog.set_response_appearance("clear", adw::ResponseAppearance::Destructive);
            let refresh = refresh.clone();
            dialog.connect_response(None, move |_, resp| {
                if resp == "clear" {
                    let (.., dir) = learned();
                    let _ = std::fs::remove_dir_all(&dir);
                    refresh();
                }
            });
            dialog.present();
        });
    }
    // Atualiza ao voltar para a página.
    summary.connect_map(move |_| refresh());
    group
}

pub fn create_page() -> gtk::Widget {
    // PreferencesPage já rola sozinha; outra ScrolledWindow em volta fazia o
    // conteúdo encolher até a largura mínima.
    let page = adw::PreferencesPage::new();
    page.set_hexpand(true);
    page.set_vexpand(true);

    // ---- Você
    let you = adw::PreferencesGroup::new();
    you.set_title("Assistente de resposta");
    you.set_description(Some(
        "Alt+K lê a conversa da janela em foco e sugere a sua próxima mensagem. O print é apagado logo depois de lido.",
    ));
    let names = adw::EntryRow::new();
    names.set_title("Como você aparece nas conversas (separe por vírgula)");
    names.set_text(&conf::get("nomes").unwrap_or_default());
    names.set_show_apply_button(true);
    names.connect_apply(|row| {
        conf::set("nomes", row.text().trim());
    });
    you.add(&names);
    page.add(&you);

    // ---- Contextos
    let ctx_group = adw::PreferencesGroup::new();
    ctx_group.set_title("Contextos");
    ctx_group.set_description(Some(
        "Instruções e informações que a IA usa para responder (tom, produtos, frete, regras).",
    ));
    let open_dir = gtk::Button::with_label("Abrir pasta");
    open_dir.add_css_class("clip-btn");
    open_dir.set_valign(gtk::Align::Center);
    open_dir.connect_clicked(|_| {
        let dir = conf::contexts_dir();
        let _ = std::fs::create_dir_all(&dir);
        conf::open(&dir);
    });
    ctx_group.set_header_suffix(Some(&open_dir));
    let rows: Rc<RefCell<Vec<adw::ActionRow>>> = Rc::default();
    fill(&ctx_group, &rows);
    page.add(&ctx_group);

    let new_group = adw::PreferencesGroup::new();
    let new_row = adw::EntryRow::new();
    new_row.set_title("Nome do novo contexto");
    new_row.set_show_apply_button(true);
    {
        let ctx_group = ctx_group.clone();
        let rows = rows.clone();
        new_row.connect_apply(move |row| {
            let name: String = row
                .text()
                .trim()
                .chars()
                .map(|c| if "/\\:*?\"<>|".contains(c) { '-' } else { c })
                .collect();
            if name.is_empty() {
                return;
            }
            let dir = conf::contexts_dir();
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join(format!("{name}.md"));
            if !path.exists() {
                let _ = std::fs::write(
                    &path,
                    "apps: \nDescreva o tom e quem você é nesta conversa.\n\nInformações que a IA pode usar:\n- \n\nRegras:\n- \n",
                );
            }
            row.set_text("");
            fill(&ctx_group, &rows);
            conf::open(&path);
        });
    }
    new_group.add(&new_row);
    page.add(&new_group);

    page.add(&learning_group(page.upcast_ref()));

    // ---- Atalhos
    let keys = adw::PreferencesGroup::new();
    keys.set_title("Atalhos");
    let open = super::atalhos::pretty(&super::atalhos::current("assistente"));
    let adjust = super::atalhos::pretty(&super::atalhos::current("ajuste"));
    keys.add(&key_row(
        &[open.as_str()],
        "Abrir o assistente",
        "Com ele aberto, gera outra resposta",
    ));
    keys.add(&key_row(
        &["Enter"],
        "Inserir a resposta no campo",
        "Só cola; você confere e envia",
    ));
    keys.add(&key_row(
        &[adjust.as_str()],
        "Escrever um ajuste",
        "Ex.: \"mais curta\" ou \"regra: nunca prometer prazo\"",
    ));
    keys.add(&key_row(&["Esc"], "Fechar", ""));
    page.add(&keys);

    // Recarrega a lista quando a página volta a aparecer (contexto criado no Alt+K).
    {
        let ctx_group = ctx_group.clone();
        let rows = rows.clone();
        page.connect_map(move |_| fill(&ctx_group, &rows));
    }

    page.upcast()
}
