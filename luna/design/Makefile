D2 := `which d2`
D2_EX := ./d2_ex.py


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


# greatfet: greatfet.d2
#	d2 --theme=$(THEME) --layout=elk $< $(basename $<).svg

# dataflow: dataflow.d2
#	d2 --theme=$(THEME) --layout=dagre $< $(basename $<).svg

# structure: structure.d2
#	d2 --theme=0 --layout=elk $< $(basename $<).svg

# facedancer: facedancer.d2
#	d2 --theme=0 --layout=elk $< $(basename $<).svg


# make greatfet
%: $@
	@echo "Do the thing: $@"
	@#d2 --theme=0 --layout=elk --watch $@.d2 $@.svg
	@#make $@.svg


# make greatfet.svg
%.svg: %.d2
	@echo "Generating: $(basename $<).svg"
	@#d2 --theme=0 --layout=elk $< $(basename $<).svg
	$(D2_EX) $< $(basename $<).svg

# watchexec --watch diagrams/ "make diagrams/top.svg"

# watch:
#	@echo "Watching for changes in: $(PWD)"
#	@watchexec "make all"



# all: top.svg test.svg structure.svg
#	@true




# Workflow:
#
# 1. Watch directory and run preprocessor to generate the composite diagram
#
#	  watchexec --watch structure/ "rm structure/top.svg ; make structure/top.svg"
#
#	  _or_
#
#	  watchexec --watch structure/ --exts d2 "./d2_ex.py structure/top.d2"
#
# 2. Watch and render preprocessor output
#
#	  d2 --theme=0 --layout=elk --watch /tmp/top.d2
