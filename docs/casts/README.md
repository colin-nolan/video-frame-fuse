The screencasts have been created with [asciinema](https://asciinema.org), and using 
[svg-term](https://github.com/marionebl/svg-term-cli) to convert them to SVGs.

Create SVG from cast:
```bash
cat mount.cast | svg-term --height 8 --width 80 --padding 10 > mount.cast.svg
 ```

Create image tile:
```bash
# montage is an ImageMagick command
montage frame-42.* -geometry +0+0 /tmp/out.png
```
