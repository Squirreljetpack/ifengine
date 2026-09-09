- [x] popup / modal discriminant in leptos on Embed & Text
timedreplace
timed stuff is a bit jank: how should it reflow elements below?
linguate can be improved by allowing to set a global fn pointer which the linguate fn just shells out to.
The default fn pointer is the current linguate fn which should become private and renamed.
Get rid of all calls to trim_lines (keep the fn tho)
one usage is allowing use of custom quote char such as '' to avoid having to use raw static strings.

save/restore
menu for switching

possibility: [[the first day]] in say p, is parsed out and creates link!("the first day", the_first_day)
link! maybe useful to support single arg "x y" -> links to x_y
embed view result pageid should not be overwritten
use ctx in egui to avoid prop drilling
simulate needs a helper that u can pass in the pageid

autostamping l! and s! is a bit weird but i think it makes a little sense