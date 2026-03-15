#!/bin/sh
mkdir -p benchdata
rsync -aR --include="*/" --include="*.html" --include="*.svg" --exclude="*" target/ benchdata/
