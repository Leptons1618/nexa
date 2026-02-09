//! Lucide-style SVG icon components.
//! Each icon is a pure component rendering an inline SVG.

use dioxus::prelude::*;

#[component]
pub fn IconMessageSquare(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" }
        }
    }
}

#[component]
pub fn IconFileText(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            line {
                x1: "16",
                y1: "13",
                x2: "8",
                y2: "13",
            }
            line {
                x1: "16",
                y1: "17",
                x2: "8",
                y2: "17",
            }
            polyline { points: "10 9 9 9 8 9" }
        }
    }
}

#[component]
pub fn IconSettings(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

#[component]
pub fn IconSend(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line {
                x1: "22",
                y1: "2",
                x2: "11",
                y2: "13",
            }
            polygon { points: "22 2 15 22 11 13 2 9 22 2" }
        }
    }
}

#[component]
pub fn IconUser(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" }
            circle { cx: "12", cy: "7", r: "4" }
        }
    }
}

#[component]
pub fn IconBot(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                x: "3",
                y: "11",
                width: "18",
                height: "10",
                rx: "2",
            }
            circle { cx: "12", cy: "5", r: "2" }
            path { d: "M12 7v4" }
            line {
                x1: "8",
                y1: "16",
                x2: "8",
                y2: "16",
            }
            line {
                x1: "16",
                y1: "16",
                x2: "16",
                y2: "16",
            }
        }
    }
}

#[component]
pub fn IconUpload(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "17 8 12 3 7 8" }
            line {
                x1: "12",
                y1: "3",
                x2: "12",
                y2: "15",
            }
        }
    }
}

#[component]
pub fn IconCheckCircle(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
            polyline { points: "22 4 12 14.01 9 11.01" }
        }
    }
}

#[component]
pub fn IconAlertCircle(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line {
                x1: "12",
                y1: "8",
                x2: "12",
                y2: "12",
            }
            line {
                x1: "12",
                y1: "16",
                x2: "12.01",
                y2: "16",
            }
        }
    }
}

#[component]
pub fn IconLoader(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line {
                x1: "12",
                y1: "2",
                x2: "12",
                y2: "6",
            }
            line {
                x1: "12",
                y1: "18",
                x2: "12",
                y2: "22",
            }
            line {
                x1: "4.93",
                y1: "4.93",
                x2: "7.76",
                y2: "7.76",
            }
            line {
                x1: "16.24",
                y1: "16.24",
                x2: "19.07",
                y2: "19.07",
            }
            line {
                x1: "2",
                y1: "12",
                x2: "6",
                y2: "12",
            }
            line {
                x1: "18",
                y1: "12",
                x2: "22",
                y2: "12",
            }
            line {
                x1: "4.93",
                y1: "19.07",
                x2: "7.76",
                y2: "16.24",
            }
            line {
                x1: "16.24",
                y1: "7.76",
                x2: "19.07",
                y2: "4.93",
            }
        }
    }
}

#[component]
pub fn IconCpu(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                x: "4",
                y: "4",
                width: "16",
                height: "16",
                rx: "2",
            }
            rect {
                x: "9",
                y: "9",
                width: "6",
                height: "6",
            }
            line {
                x1: "9",
                y1: "1",
                x2: "9",
                y2: "4",
            }
            line {
                x1: "15",
                y1: "1",
                x2: "15",
                y2: "4",
            }
            line {
                x1: "9",
                y1: "20",
                x2: "9",
                y2: "23",
            }
            line {
                x1: "15",
                y1: "20",
                x2: "15",
                y2: "23",
            }
            line {
                x1: "20",
                y1: "9",
                x2: "23",
                y2: "9",
            }
            line {
                x1: "20",
                y1: "14",
                x2: "23",
                y2: "14",
            }
            line {
                x1: "1",
                y1: "9",
                x2: "4",
                y2: "9",
            }
            line {
                x1: "1",
                y1: "14",
                x2: "4",
                y2: "14",
            }
        }
    }
}

#[component]
pub fn IconDatabase(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            ellipse {
                cx: "12",
                cy: "5",
                rx: "9",
                ry: "3",
            }
            path { d: "M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" }
            path { d: "M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" }
        }
    }
}

#[component]
pub fn IconTag(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" }
            line {
                x1: "7",
                y1: "7",
                x2: "7.01",
                y2: "7",
            }
        }
    }
}

#[component]
pub fn IconTrash(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "3 6 5 6 21 6" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
            line {
                x1: "10",
                y1: "11",
                x2: "10",
                y2: "17",
            }
            line {
                x1: "14",
                y1: "11",
                x2: "14",
                y2: "17",
            }
        }
    }
}

#[component]
pub fn IconChevronUp(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "18 15 12 9 6 15" }
        }
    }
}

#[component]
pub fn IconChevronDown(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}

#[component]
pub fn IconCopy(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                x: "9",
                y: "9",
                width: "13",
                height: "13",
                rx: "2",
            }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

#[component]
pub fn IconExternalLink(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
            polyline { points: "15 3 21 3 21 9" }
            line {
                x1: "10",
                y1: "14",
                x2: "21",
                y2: "3",
            }
        }
    }
}

#[component]
pub fn IconX(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line {
                x1: "18",
                y1: "6",
                x2: "6",
                y2: "18",
            }
            line {
                x1: "6",
                y1: "6",
                x2: "18",
                y2: "18",
            }
        }
    }
}

#[component]
pub fn IconFilePdf(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            path { d: "M10 12l-2 4h4l-2 4" }
        }
    }
}

#[component]
pub fn IconFileCode(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            path { d: "m10 13-2 2 2 2" }
            path { d: "m14 17 2-2-2-2" }
        }
    }
}

#[component]
pub fn IconPanelLeft(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                x: "3",
                y: "3",
                width: "18",
                height: "18",
                rx: "2",
            }
            line {
                x1: "9",
                y1: "3",
                x2: "9",
                y2: "21",
            }
        }
    }
}

#[component]
pub fn IconChevronLeft(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "15 18 9 12 15 6" }
        }
    }
}

#[component]
pub fn IconChevronRight(size: Option<u32>) -> Element {
    let s = size.unwrap_or(20);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{s}",
            height: "{s}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "9 18 15 12 9 6" }
        }
    }
}
