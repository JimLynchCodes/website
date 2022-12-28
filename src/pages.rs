mod index;
mod join;
use crate::{
    fonts,
    html::Classes,
    markdown::to_html,
    mobs::{self, Event, Mob, MobParticipant},
    style, COMMIT_HASH, DESCRIPTION, GITHUB_ORGANIZATION_URL, NAME, REPO_URL, ZULIP_URL,
};
use chrono::Utc;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use ssg::{Asset, Source, Targets};
use std::{path::Path, vec};

pub(crate) fn base(
    title: String,
    content: Markup,
    stylesheets: impl IntoIterator<Item = String>,
    content_classes: Classes,
    targets: &Targets,
) -> Markup {
    let version = Utc::now().timestamp_millis();
    let content_classes = content_classes + classes!["grow" "flex" "flex-col" "justify-center"];
    const NAV_ICON_SIZE: u8 = 32;
    let markup = html! {
      (DOCTYPE)
      html lang="en" class=(classes![format!("font-[{}]", fonts::VOLLKORN) "[font-size:16px]" format!("bg-{}", style::BACKGROUND_COLOR) format!("text-{}", style::TEXT_COLOR)]) {
        head {
          title { (format!("{title}; {NAME}")) }
          meta charset="utf-8";
          meta description=(DESCRIPTION);
          meta name="viewport" content="width=device-width, initial-scale=1.0";
          link rel="stylesheet" href={ "/index.css?v=" (version) };
          @for stylesheet in stylesheets {
              link rel="stylesheet" href=(stylesheet);
          }
          style {
            // TODO extract as font utility
            @for font in fonts::ALL {(PreEscaped(format!("
              @font-face {{
                font-family: '{}';
                src: url('/{}') format('truetype');
              }}
            ", font.name, fonts::output_filename(&font))))}
          }
        }
        body class=(classes!("min-h-screen" "py-1" "px-1" "md:px-5" "flex" "flex-col" "gap-1" "max-w-screen-xl" "mx-auto")) {
            div class=(classes!("flex" "justify-between" "items-center" "flex-wrap" "gap-x-2" "gap-y-1" "uppercase" "text-lg")) {
                div class=(classes!("flex" "flex-col" "gap-x-2" "whitespace-nowrap")) {
                    p class=(classes!("tracking-widest" "text-center")) { (NAME) }
                    p class=(classes!("text-sm" "opacity-75")) { (DESCRIPTION) }
                }
                div class=(classes!("flex" "flex-wrap" "gap-x-2")) {
                    a href=(targets.relative("index.html").unwrap().to_str().unwrap()) { "Calendar" }
                    a href=(targets.relative("join.html").unwrap().to_str().unwrap()) { "Join" }
                }
                div class=(classes!("flex" "items-center" "gap-x-2")) {
                    a href=(ZULIP_URL.to_string()) {
                        img
                            width=(NAV_ICON_SIZE)
                            alt="Zulip"
                            src=(targets.relative("zulip_logo.svg").unwrap().to_str().unwrap());
                    }
                    a class=(classes!("invert")) href=(GITHUB_ORGANIZATION_URL.to_string()) {
                        img
                            width=(NAV_ICON_SIZE)
                            alt="GitHub"
                            src=(targets.relative("inverticat.svg").unwrap().to_str().unwrap());
                    }
                    a href="https://twitter.com/mobusoperandi" {
                        img
                            width=(NAV_ICON_SIZE)
                            alt="Twitter"
                            src=(targets.relative("twitter_logo.svg").unwrap().to_str().unwrap());
                    }
                }
            }
            hr {}
            div class=(content_classes) {
                (content)
            }
            hr {}
            div class=(classes!("flex" "justify-between" "flex-wrap" "items-end")) {
                pre class=(classes!("text-xs")) { code { (*COMMIT_HASH) } }
                a class=(classes!("text-sm")) href=(REPO_URL.to_string()) { "Source"}
            }
        }
      }
    };
    markup
}

pub(crate) fn mob_page(mob: Mob) -> Asset {
    let id = mob.id.clone();
    Asset::new(
        ["mobs", &format!("{id}.html")].into_iter().collect(),
        async move {
            Source::BytesWithAssetSafety(Box::new(move |targets| {
                let (calendar_html, calendar_stylesheet) =
                    calendar(&targets, mob.events(&targets, false));
                Ok(base(
                    mob.title.clone(),
                    html! {
                        div class=(classes!("sm:grid" "grid-cols-2" "text-center" "tracking-wide")) {
                            div class=(classes!("py-12")) {
                                h1 class=(classes!("text-4xl")) { (mob.title) }
                                @if let Some(subtitle) = mob.subtitle {
                                    p { (subtitle) }
                                }
                            }
                            div class=(classes!("py-12")) {
                                h2 { "Participants" }
                                div class=(classes!("font-bold")) {
                                    @for mob_participant in mob.participants {
                                        @match mob_participant {
                                            MobParticipant::Hidden => div { "(Anonymous participant)" },
                                            MobParticipant::Public(person) => a class=(classes!("block")) href=(person.social_url.to_string()) { (person.name) },
                                        }
                                    }
                                }
                            }
                        }
                        div class=(*style::PROSE_CLASSES) {
                            (PreEscaped(to_html(&mob.freeform_copy_markdown)))
                        }
                        hr {}
                        (calendar_html)
                    },
                    [calendar_stylesheet],
                    classes!("gap-6"),
                    &targets,
                )
                .0
                .into_bytes())
            }))
        },
    )
}

pub(crate) async fn all() -> Vec<Asset> {
    let mobs = mobs::read_all_mobs().await;
    let mut mob_pages = mobs.iter().cloned().map(mob_page).collect::<Vec<_>>();
    let mut pages = vec![index::page().await, join::page()];
    pages.append(&mut mob_pages);
    pages
}

pub(crate) fn calendar(targets: &Targets, events: Vec<Event>) -> (Markup, String) {
    let events = serde_json::to_string(&events).unwrap();
    let html = html! {
        div class=(classes!("[--fc-page-bg-color:transparent]")) {}
        script defer src=(targets.relative(Path::new("fullcalendar.js")).unwrap().display().to_string()) {}
        script {
            (PreEscaped(format!("window.addEventListener('DOMContentLoaded', () => {{
                const events = JSON.parse('{events}')
                {}
            }})", include_str!("pages/calendar.js"))))
        }
    };
    let stylesheet = targets
        .relative("fullcalendar.css")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    (html, stylesheet)
}
