use crate::seam_carving::resize;
use image::FilterType;
use std::time::Duration;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, ImageData};
use stdweb::*;
use yew::services::interval::IntervalTask;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::{IntervalService, RenderService};
use yew::{html, ChangeData, Component, ComponentLink, Html, Renderable, ShouldRender};

pub struct Model {
    interval: IntervalService,
    interval_task: IntervalTask,
    renderer: RenderService,
    link: ComponentLink<Model>,
    reader: ReaderService,
    reader_task: Vec<ReaderTask>,
    file: Option<FileData>,
    by_chunks: bool,
    cur_count: u32,
}

pub enum Msg {
    Loaded(FileData),
    Files(Vec<File>),
    Tick,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        let interval_task = interval.spawn(Duration::from_millis(2), link.send_back(|_| Msg::Tick));

        Model {
            interval,
            interval_task,
            renderer: RenderService::new(),
            reader: ReaderService::new(),
            link,
            reader_task: vec![],
            file: None,
            by_chunks: false,
            cur_count: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Loaded(file) => {
                let mut ctx = js! {
                    var canvas = document.getElementById("imageCanvas");
                    var ctx = canvas.getContext("2d");
                    return ctx;
                };

                let reference = match ctx {
                    stdweb::Value::Reference(reference) => reference,
                    _ => unreachable!(),
                };

                let ctx: CanvasRenderingContext2d = reference.try_into().unwrap();

                match image::load_from_memory(&file.content) {
                    Ok(image) => {
                        //let resized = image.resize_exact(300, 300, FilterType::Nearest);

                        let converted = resize(&image, 300, 300).to_rgba();

                        let serialized = converted.into_raw();

                        // let data = vec![128u8; 300 * 300 * 4];
                        let data = serialized;

                        let image_data: ImageData = js! {
                            var id = new ImageData(new Uint8ClampedArray(@{&data}), 300, 300);
                            return id;
                        }
                        .try_into()
                        .unwrap();

                        ctx.put_image_data(image_data, 0.0, 0.0).unwrap();
                    }
                    Err(e) => eprintln!("error: {}", e),
                }

                self.file = Some(file);
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let task = {
                        let callback = self.link.send_back(Msg::Loaded);
                        self.reader.read_file(file, callback)
                    };
                    self.reader_task.push(task);
                }
            }
            Msg::Tick => {
                self.cur_count += 1;
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let flag = self.by_chunks;
        html! {
            <div>
             <h1 id="heading",>{ "Seam carving" }</h1>
                <div id="wrapper",>
                    <div>
                        <input type="file", multiple=false, onchange=|value| {
                                let mut result = Vec::new();
                                if let ChangeData::Files(files) = value {
                                    result.extend(files);
                                }
                                Msg::Files(result)
                            },/>
                    </div>
                    <canvas id="imageCanvas", width="300", height="300",/>
                    <div>
                        { self.view_file(self.file.as_ref()) }
                    </div>
                    <div>
                        { self.cur_count }
                    </div>
                 </div>
            </div>
        }
    }
}

impl Model {
    fn view_file(&self, data: Option<&FileData>) -> Html<Self> {
        if let Some(data) = data {
            html! {
                <li>{ format!("{}: {} bytes", data.name, data.content.len()) }</li>
            }
        } else {
            html! {
                <li></li>
            }
        }
    }
}
