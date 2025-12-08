
pub struct View {
	objects: Vec<Object>
}

// Turns into a div
Object {
	Paragraph (Vec<Element>)
	Choice
	Image
}

Element {
	Modifiers (bold, dim, italic, super, sub)
	Style: Hashmap<String, String>
	Classes: Vec<String>
	Action(Action)
}

type GlobalState = HashMap<String,Int>

let app = App::new
let global_dict = GlobalState::new()
let mut widget : impl Page = initial_widget()
loop {
   match widget(&mut app, state) {
   Render(view: Page) => {
         // render this widget
   }
   Next(fn) => {
	widget = fn
   }
   Back(n) => todo!()!;
   TunnelBack => todo()!;
}

enum Next {
	Render(View),
	Back(n),
	Next(fn),
	TunnelBack
}

type Page = Fn(App, GlobalState) -> Next