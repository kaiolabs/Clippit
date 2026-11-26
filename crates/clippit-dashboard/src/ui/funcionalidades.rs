//! Funcionalidades: tudo o que o Clippit faz, como funciona e como usar.
//! Cada item abre para mostrar os passos e as teclas. Tem busca no topo.

use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

struct Feature {
    titulo: &'static str,
    resumo: &'static str,
    teclas: &'static [&'static str],
    passos: &'static [&'static str],
}

const GRUPOS: &[(&str, &[Feature])] = &[
    ("Lançador", &[
        Feature {
            titulo: "Lançador",
            resumo: "Apps, arquivos, sites e comandos num lugar só",
            teclas: &["{lancador}"],
            passos: &[
                "Aperte {lancador} e digite o nome de um app, de um arquivo ou um site.",
                "Enter abre. Esc fecha. Clicar fora volta para a janela de trás sem perder o que você digitou.",
                "A seta ← (ou Backspace com o campo vazio) volta para o início.",
            ],
        },
        Feature {
            titulo: "Contas, conversões e atalhos com número",
            resumo: "Resolve na hora, sem abrir nada",
            teclas: &[],
            passos: &[
                "Contas: \"340 + 15%\", \"(1200 - 350) / 3\".",
                "Conversões: \"10 km em milhas\", \"2h30 em minutos\", \"100 °F em °C\".",
                "Com número: \"volume 40\", \"brilho 70\", \"timer 25\" (avisa em 25 minutos).",
            ],
        },
        Feature {
            titulo: "Controles e janelas",
            resumo: "Volume, brilho, Wi-Fi, Bluetooth, tema e a janela de trás",
            teclas: &[],
            passos: &[
                "Digite \"wi-fi\", \"bluetooth\", \"modo escuro\", \"não perturbe\" ou \"mudo\".",
                "Para a janela que estava aberta: \"esquerda\", \"direita\", \"maximizar\", \"centralizar\", \"outro monitor\".",
                "Sistema: \"bloquear\", \"suspender\", \"esvaziar lixeira\"; reiniciar e desligar pedem confirmação.",
            ],
        },
        Feature {
            titulo: "Emoji e símbolos",
            resumo: "Grade de emoji com busca em português",
            teclas: &[":"],
            passos: &[
                "No lançador, digite : e o nome (\":coração\", \":fogo\") ou procure \"emoji\".",
                "Enter cola o emoji na janela de trás. Os que você mais usa aparecem primeiro.",
            ],
        },
        Feature {
            titulo: "Snippets (textos prontos)",
            resumo: "Um gatilho curto vira um texto inteiro, em qualquer campo",
            teclas: &[";"],
            passos: &[
                "Em qualquer campo de texto, digite o gatilho (ex.: ;email) e ele vira o texto salvo.",
                "No lançador, ; mostra a lista de snippets.",
                "Para criar: peça à IA (\"cria um snippet ;pix com a minha chave\"). Aceita {data}, {hora} e {area} (o que está copiado).",
            ],
        },
    ]),
    ("Histórico do que você copia", &[
        Feature {
            titulo: "Histórico",
            resumo: "Tudo o que você copiou (textos, links, código e imagens)",
            teclas: &["{historico}"],
            passos: &[
                "{historico} abre o histórico em qualquer lugar. Digite para filtrar.",
                "Enter cola na janela de trás; Ctrl+Enter só copia.",
                "Ctrl+P fixa no topo (não some com o tempo); Ctrl+Del apaga; Ctrl+Espaço amplia a prévia.",
            ],
        },
        Feature {
            titulo: "Transformar antes de colar",
            resumo: "MAIÚSCULAS, uma linha só, só os números e mais",
            teclas: &["Tab"],
            passos: &[
                "No histórico, escolha o item e aperte Tab.",
                "Escolha a transformação; o resultado é colado na janela de trás.",
            ],
        },
        Feature {
            titulo: "Prints pesquisáveis",
            resumo: "O texto dos prints e imagens copiadas é lido sozinho",
            teclas: &[],
            passos: &[
                "Copie ou tire um print: o Clippit lê o texto dele (OCR) no fundo.",
                "Depois é só procurar no histórico por uma palavra que estava na imagem.",
                "Liga, desliga e escolhe os idiomas em Geral → Imagens.",
            ],
        },
    ]),
    ("IA local", &[
        Feature {
            titulo: "Chat com a IA",
            resumo: "Roda no seu computador; nada vai para a nuvem",
            teclas: &["{ia}"],
            passos: &[
                "{ia} abre o chat (ou digite ? no lançador).",
                "@ anexa um app, um arquivo ou um print; Alt+V fala a pergunta.",
                "Ctrl+N começa outra conversa; Ctrl+H mostra as anteriores.",
                "Esc para a resposta no meio; o segundo Esc fecha.",
            ],
        },
        Feature {
            titulo: "O que a IA consegue fazer",
            resumo: "Ela usa ferramentas de verdade, não só conversa",
            teclas: &[],
            passos: &[
                "Internet: pesquisar, notícias, cotação, clima e ler páginas.",
                "Computador: abrir apps e sites, mexer nas janelas, volume, brilho, Wi-Fi, música, tema e modo de energia.",
                "Arquivos: achar, ler, organizar, juntar/separar PDF, converter e diminuir imagens, ver o que enche o disco.",
                "Trabalho: ver projetos no git, abrir projeto no Cursor, rodar build/teste no fundo e avisar quando acabar.",
                "Dia a dia: contas exatas, unidades, tradução, notas, lembretes, procurar no que você copiou e colar texto no campo.",
                "Na tela: \"o que diz esse erro?\" — ela tira um print e lê.",
            ],
        },
        Feature {
            titulo: "Comandos com confirmação",
            resumo: "Nada que muda o seu computador roda sem você ver",
            teclas: &["Enter"],
            passos: &[
                "Quando a IA precisa rodar algo que muda o sistema, ela mostra o comando e espera.",
                "Enter (ou Rodar) executa; escrever outra coisa cancela.",
                "Só leitura (ver memória, espaço, processos) roda direto.",
            ],
        },
        Feature {
            titulo: "Ações no texto selecionado",
            resumo: "Resumir, traduzir, explicar ou reescrever o que você selecionou",
            teclas: &["{acoes}"],
            passos: &[
                "Selecione um texto em qualquer app e aperte {acoes} (sem seleção, usa o que está copiado).",
                "Escolha: resumir, traduzir, explicar, mais formal, mais casual, mais curto ou responder.",
            ],
        },
        Feature {
            titulo: "Ditado",
            resumo: "Fale em vez de digitar",
            teclas: &["Alt+V"],
            passos: &["No chat da IA, aperte Alt+V, fale e aperte de novo. A voz vira texto no seu computador."],
        },
    ]),
    ("Escrita", &[
        Feature {
            titulo: "Corretor em qualquer campo",
            resumo: "Um selo aparece ao lado do campo quando há algo a corrigir",
            teclas: &["{sugestoes}", "{aplicar}"],
            passos: &[
                "Escreva normalmente; o selo mostra quantas sugestões há.",
                "{sugestoes} abre as sugestões; {aplicar} aplica todas (Ctrl+S também, fora de editores).",
                "Ligar, desligar e onde rodar ficam na tela Corretor.",
            ],
        },
        Feature {
            titulo: "Aprende com você",
            resumo: "Trocas que você aceita viram automáticas",
            teclas: &[],
            passos: &[
                "Uma troca aceita duas vezes passa a sair sozinha.",
                "Palavras que você ignora (nomes, gírias) entram no seu dicionário.",
                "Dá para ver e apagar tudo na tela Assistente → Aprendizado.",
            ],
        },
        Feature {
            titulo: "Assistente de resposta",
            resumo: "Lê a conversa da janela (WhatsApp, Slack…) e sugere a resposta",
            teclas: &["{assistente}"],
            passos: &[
                "Com a conversa aberta, aperte {assistente}: aparecem 3 opções (curta, normal, completa).",
                "1, 2 ou 3 escolhe; Enter insere no campo; {ajuste} pede um ajuste (\"mais curta\").",
                "Contextos (tom, produtos, frete) ficam na tela Assistente.",
            ],
        },
    ]),
    ("Sistema", &[
        Feature {
            titulo: "Modo jogo",
            resumo: "Tira a IA da placa de vídeo para os jogos",
            teclas: &["{modo_jogo}"],
            passos: &[
                "Jogos abertos pela Steam, Proton ou Wine são detectados e a IA sai da placa sozinha.",
                "{modo_jogo} liga e desliga na mão. Os jogos já abrem na placa NVIDIA.",
            ],
        },
        Feature {
            titulo: "Doutor do Clippit",
            resumo: "Acha e corrige sozinho o que parar de funcionar",
            teclas: &[],
            passos: &[
                "A cada 2 minutos ele confere a extensão, o histórico, a IA, os dados e os atalhos.",
                "O que é seguro ele corrige e avisa com uma notificação.",
                "O resto vira uma sugestão no chat da IA, esperando o seu Rodar.",
                "Pergunte à IA \"o Clippit tá com algum problema?\" para um diagnóstico na hora.",
            ],
        },
        Feature {
            titulo: "Privacidade",
            resumo: "Tudo fica no seu computador",
            teclas: &[],
            passos: &[
                "Histórico, IA, corretor, voz e leitura de imagens rodam localmente.",
                "A internet só é usada quando você pede (pesquisa, notícias, cotação, clima).",
                "Arquivos com senhas e chaves não são lidos pela IA.",
                "Dá para desligar a IA toda, ou só algumas funções, em Geral → Inteligência artificial.",
            ],
        },
    ]),
];

/// Troca {ia}, {lancador}… pelo atalho que o usuário configurou (Atalhos).
fn resolve(t: &str) -> String {
    let mut out = t.to_string();
    if out.contains("{historico}") {
        out = out.replace("{historico}", &history_key());
    }
    for (id, ..) in super::atalhos::SHORTCUTS {
        let tag = format!("{{{id}}}");
        if out.contains(&tag) {
            out = out.replace(&tag, &super::atalhos::pretty(&super::atalhos::current(id)));
        }
    }
    out
}

/// O atalho do histórico vem do GNOME (config.toml), não do corretor.conf.
fn history_key() -> String {
    let cfg = clippit_core::Config::load().unwrap_or_default();
    let m = match cfg.hotkeys.show_history_modifier.to_lowercase().as_str() {
        "ctrl" | "control" => "Ctrl".to_string(),
        "super" | "meta" => "Super".into(),
        "shift" => "Shift".into(),
        "alt" => "Alt".into(),
        o => o.to_string(),
    };
    format!("{m}+{}", cfg.hotkeys.show_history_key.to_uppercase())
}

type Live = Rc<RefCell<Vec<(gtk::Label, &'static str)>>>;

fn keycaps(keys: &[&'static str], live: &Live) -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    b.set_valign(gtk::Align::Center);
    for k in keys {
        let l = gtk::Label::new(Some(&resolve(k)));
        l.add_css_class("clip-key");
        live.borrow_mut().push((l.clone(), k));
        b.append(&l);
    }
    b
}

pub fn create_page() -> gtk::Widget {
    let page = adw::PreferencesPage::new();
    // Textos com atalho: refeitos toda vez que a tela aparece (pega trocas feitas em Atalhos).
    let live: Live = Rc::new(RefCell::new(Vec::new()));

    // Busca
    let search_group = adw::PreferencesGroup::new();
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Procurar uma funcionalidade ou tecla…"));
    search.add_css_class("clip-search");
    search_group.add(&search);
    page.add(&search_group);

    let mut rows: Vec<(adw::ExpanderRow, String, usize)> = Vec::new();
    let mut groups: Vec<adw::PreferencesGroup> = Vec::new();
    for (gi, (grupo, features)) in GRUPOS.iter().enumerate() {
        let g = adw::PreferencesGroup::new();
        g.set_title(grupo);
        for f in *features {
            let row = adw::ExpanderRow::new();
            row.set_title(f.titulo);
            row.set_subtitle(f.resumo);
            // add_suffix só existe na libadwaita 1.4+; add_action põe à direita.
            let right = keycaps(f.teclas, &live);
            // Seta nossa: a da libadwaita usa um ícone que nem todo tema tem.
            let arrow = gtk::Image::from_icon_name("pan-down-symbolic");
            arrow.add_css_class("clip-arrow");
            right.append(&arrow);
            row.add_action(&right);
            row.connect_expanded_notify(move |r| {
                arrow.set_icon_name(Some(if r.is_expanded() {
                    "pan-up-symbolic"
                } else {
                    "pan-down-symbolic"
                }));
            });
            let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
            body.set_margin_start(14);
            body.set_margin_end(14);
            body.set_margin_top(8);
            body.set_margin_bottom(12);
            for (i, p) in f.passos.iter().enumerate() {
                let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                let n = gtk::Label::new(Some(&format!("{}", i + 1)));
                n.add_css_class("clip-step");
                n.set_valign(gtk::Align::Start);
                let t = gtk::Label::new(Some(&resolve(p)));
                live.borrow_mut().push((t.clone(), p));
                t.set_wrap(true);
                t.set_xalign(0.0);
                t.set_hexpand(true);
                t.add_css_class("clip-step-text");
                line.append(&n);
                line.append(&t);
                body.append(&line);
            }
            let holder = gtk::ListBoxRow::new();
            holder.set_activatable(false);
            holder.set_selectable(false);
            holder.set_child(Some(&body));
            row.add_row(&holder);
            let hay = format!(
                "{} {} {} {}",
                f.titulo,
                f.resumo,
                f.teclas.join(" "),
                f.passos.join(" ")
            )
            .to_lowercase();
            g.add(&row);
            rows.push((row, hay, gi));
        }
        page.add(&g);
        groups.push(g);
    }

    search.connect_search_changed(move |s| {
        let q = s.text().to_lowercase();
        let words: Vec<&str> = q.split_whitespace().collect();
        let mut any = vec![false; groups.len()];
        for (row, hay, gi) in &rows {
            let show = words.iter().all(|w| hay.contains(w));
            row.set_visible(show);
            row.set_expanded(show && !words.is_empty());
            any[*gi] |= show;
        }
        for (g, show) in groups.iter().zip(any) {
            g.set_visible(show);
        }
    });

    page.connect_map(move |_| {
        for (label, template) in live.borrow().iter() {
            label.set_text(&resolve(template));
        }
    });

    page.upcast()
}
