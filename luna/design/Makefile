# https://d2lang.com/tour/layouts/
LAYOUT := dagre
#LAYOUT := elk

# https://github.com/terrastruct/d2/tree/master/d2themes
#THEME := 0 # Default
THEME := 103 # Earth tones

# greatfet: greatfet.d2
#	d2 --theme=$(THEME) --layout=elk --watch $< $(basename $<).svg

# dataflow: dataflow.d2
#	d2 --theme=$(THEME) --layout=dagre --watch $< $(basename $<).svg

# structure: structure.d2
#	d2 --theme=0 --layout=elk --watch $< $(basename $<).svg

# facedancer: facedancer.d2
#	d2 --theme=0 --layout=elk --watch $< $(basename $<).svg


greatfet: greatfet.d2
	d2 --theme=$(THEME) --layout=elk $< $(basename $<).svg

dataflow: dataflow.d2
	d2 --theme=$(THEME) --layout=dagre $< $(basename $<).svg

structure: structure.d2
	d2 --theme=0 --layout=elk $< $(basename $<).svg

facedancer: facedancer.d2
	d2 --theme=0 --layout=elk $< $(basename $<).svg


# all: top.svg test.svg structure.svg
#	@true

# %: $@
#	@echo "Waching: $@"
#	d2 --theme=0 --layout=elk --watch $@.d2 $@.svg

# %.svg: %.d2
#	@echo "input: $<  output: $@"
#	@# @echo "Generate: $(basename $<).svg"
#	@# @touch $(basename $<).svg

# watch:
#	@echo "Watching for changes in: $(PWD)"
#	@watchexec "make all"
