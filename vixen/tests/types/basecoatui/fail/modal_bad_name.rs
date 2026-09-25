use vixen::ui::basecoatui::Drawer;

const DRAWER: Drawer = Drawer::new("a b");

fn main() {
    let _ = DRAWER.shell();
}
