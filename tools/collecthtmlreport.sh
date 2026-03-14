#!/bin/sh
mkdir -p benchdata
rsync -aR --include="*/" --include="*.html" --exclude="*" target/ benchdata/
