#!/bin/bash

cd ~/dev || exit
git clone ssh://aur@aur.archlinux.org/lnr-bin.git
cd lnr-bin || exit
git remote add aur ssh://aur@aur.archlinux.org/lnr-bin.git
cd ../lnr || exit
