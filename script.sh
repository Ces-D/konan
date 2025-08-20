#!/bin/bash

function run_mobile() {
  open /Applications/Xcode.app/Contents/Developer/Applications/Simulator.app && dx serve --platform ios --package konan_mobile
}

"$@" # Calls whatever function name you pass in
