#!/bin/sh
cd ../lnr-bin/
git pull
cd ../lnr/
makepkg --printsrcinfo > ../lnr-bin/.SRCINFO
mv PKGBUILD ../lnr-bin/
rm *.tar.gz
cd ../lnr-bin/
git add .
git commit -m "new version"
git push aur
cd ../lnr