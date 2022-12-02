# https://d2lang.com/tour/layouts/
LAYOUT := dagre
#LAYOUT := elk

# https://github.com/terrastruct/d2/tree/master/d2themes
#THEME := 0 # Default
THEME := 103 # Earth tones

dataflow: dataflow.d2
	d2 --theme=$(THEME) --layout=$(LAYOUT) --watch $< $(basename $<).svg

greatfet: greatfet.d2
	d2 --theme=$(THEME) --layout=elk --watch $< $(basename $<).svg
