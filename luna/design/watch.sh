#!/usr/bin/env zsh

fn_watchexec () {
    watchexec --watch $1/  --exts d2 "./d2_ex.py $1/top_$1.d2"
}

fn_d2 () {
    d2 --theme=$1 --layout=$2 --watch /tmp/top_$3.d2
}


trap "kill %1; kill %2; kill %3; kill %4; kill %5; kill %6; kill %7; kill %8" SIGINT

fn_watchexec "dataflow"   & fn_d2 "103" "dagre" "dataflow"   & \
fn_watchexec "facedancer" & fn_d2 "0"   "elk"   "facedancer" & \
fn_watchexec "greatfet"   & fn_d2 "0"   "elk"   "greatfet"   & \
fn_watchexec "structure"  & fn_d2 "0"   "elk"   "structure"  & \
wait


# https://d2lang.com/tour/layouts/
# https://github.com/terrastruct/d2/tree/master/d2themes
#D2_LAYOUT := dagre or elk
#D2_THEME := 0 or 103 # Earth tones
