#!/bin/bash

probe-rs download $1 $2 $3
../../probe-plotter-tools/target/release/viewer $1 $3
