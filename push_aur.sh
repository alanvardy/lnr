#!/bin/bash

cd ../lnr-bin/ || exit
git pull
cd ../lnr/ || exit
mv target/cargo-aur/PKGBUILD ./PKGBUILD
makepkg --printsrcinfo > ../lnr-bin/.SRCINFO
mv PKGBUILD ../lnr-bin/
rm target/cargo-aur/*.tar.gz
cd ../lnr-bin/ || exit
git add .
git commit -m "new version"
git push aur
cd ../lnr || exit
