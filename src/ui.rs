use std::{any::Any, sync::Arc};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait UiElement<P>: AsAny {
    fn render(&self, render_pass: &mut wgpu::RenderPass);
    fn prerender(&mut self, _queue: &wgpu::Queue, _params: Arc<P>, _buffer: &[f32]) {}

    fn z_layer(&self) -> i32 {
        0
    }
}

pub trait UiBox {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn position(&self) -> (u16, u16);

    fn get_vertices<const VIEW_WIDTH: usize, const VIEW_HEIGHT: usize>(&self) -> [f32; 12] {
        let view_w = VIEW_WIDTH as f32;
        let view_h = VIEW_HEIGHT as f32;

        let (x, y) = self.position();
        let x1 = (x as f32 / view_w) * 2.0 - 1.0;
        let y1 = 1.0 - (y as f32 / view_h) * 2.0;

        let x2 = (x + self.width()) as f32 / view_w * 2.0 - 1.0;
        let y2 = 1.0 - (y + self.height()) as f32 / view_h * 2.0;

        [
            x1, y1, x2, y1, x1, y2, //
            x1, y2, x2, y1, x2, y2,
        ]
    }
}

impl UiBox for [u16; 4] {
    fn width(&self) -> u16 {
        self[2] - self[0]
    }

    fn height(&self) -> u16 {
        self[3] - self[1]
    }

    fn position(&self) -> (u16, u16) {
        (self[0], self[1])
    }
}

pub trait UiInteractive<P>: UiElement<P> + UiBox {
    fn is_mouse_over(&self, mouse_pos: (i16, i16)) -> bool {
        let (x, y) = self.position();
        let (mouse_x, mouse_y) = mouse_pos;
        let mouse_x = mouse_x as u16;
        let mouse_y = mouse_y as u16;

        mouse_x >= x && mouse_x < x + self.width() && mouse_y >= y && mouse_y < y + self.height()
    }
}

pub struct UiCollection<P> {
    scene_elements: Vec<Box<dyn UiElement<P>>>,
}

impl<P> UiCollection<P> {
    pub fn new() -> Self {
        Self {
            scene_elements: Vec::new(),
        }
    }

    pub fn append<E>(&mut self, element: E)
    where
        E: UiElement<P> + 'static,
    {
        self.scene_elements.push(Box::new(element));
    }

    pub fn batch_append(&mut self, elements: Vec<Box<dyn UiElement<P>>>) {
        self.scene_elements.extend(elements);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn UiElement<P>>> {
        let mut elems: Vec<&Box<dyn UiElement<P>>> = self.scene_elements.iter().collect();
        elems.sort_by_key(|e| e.z_layer());
        elems.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn UiElement<P>>> {
        let mut elems: Vec<&mut Box<dyn UiElement<P>>> = self.scene_elements.iter_mut().collect();
        elems.sort_by_key(|e| e.z_layer());
        elems.into_iter()
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Box<dyn UiElement<P>>) -> bool,
    {
        self.scene_elements.retain(|e| f(e));
    }
}
