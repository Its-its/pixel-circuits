@echo off

start wt --title Frontend -d ../frontend cmd.exe /k npm run-script build ;^
 split-pane --title Backend -d ../backend cmd.exe /k cargo run ;^
 split-pane -H --title SCSS -d .. cmd.exe /k sass --watch ./app/scss/main.scss ./frontend/public/css/main.css ;^
 focus-tab -t 0