D2 := `which d2`
D2_EX := ./d2_ex.py
MARKDOWN := multimarkdown

preview:
	$(MARKDOWN) --nosmart rfp.md -o rfp.html
	open rfp.html

watch:
	@echo "Watching for changes ..."
	./watch.sh

usb:
	d2 --watch=true usb/top.py

# Workflow:
#
# 1. Watch directory and run preprocessor to generate the composite diagram
#
#	  watchexec --watch structure/ "rm structure/top.svg ; make structure/top.svg"
#
#	  _or_
#
#	  watchexec --watch dataflow/  --exts d2 "./d2_ex.py dataflow/top_dataflow.d2"
#	  watchexec --watch structure/ --exts d2 "./d2_ex.py structure/top_structure.d2"
#
# 2. Watch and render preprocessor output
#
#	  d2 --theme=103 --layout=dagre --watch /tmp/top_dataflow.d2
#	  d2 --theme=0	 --layout=elk	--watch /tmp/top_structure.d2


# # make greatfet
# %: $@
#	@echo "Do the thing: $@"
#	@#d2 --theme=0 --layout=elk --watch $@.d2 $@.svg
#	@#make $@.svg

# # make greatfet.svg
# %.svg: %.d2
#	@echo "Generating: $(basename $<).svg"
#	@#d2 --theme=0 --layout=elk $< $(basename $<).svg
#	$(D2_EX) $< $(basename $<).svg
