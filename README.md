# Aquiver
Playing video on Minecraft Bedrock.

## Building
1.Install Rust.

2.Open Terminal(cmd), clone(download) this project.
```
cd Aquiver
cargo build --release
```

## USAGE:
> aquiver [OPTIONS]

## FLAGS:
```shell script
        --help       Prints help information
    -V, --version    Prints version information
```

### OPTIONS:
    - d, --description <description>    Descriptions
    - h, --height <height>              The video's height (float)
    - l, --loop <loop>                  Automatically replay the video
    - m, --mode <mode>                  Face camera mode (look_xyz, rotate_xyz etc.)
    - n, --name <name>                  Resource pack's name(String)
    - p, --path <path>                  The path of the video(GIF)
    - w, --width <width>                The video's width (float)


## How to use?
Open Minecraft, load the pack and enter the world.
Run chat command:
```
function init
```
Place an armor_stand named {packName}.

Give yourself a repeating command block.Input this command into the command block, set at running.
```
function loop
```

## LICENSE  ![NY NC ND](https://i.creativecommons.org/l/by-nc-nd/4.0/88x31.png)
This work is licensed under a [Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International License.](http://creativecommons.org/licenses/by-nc-nd/4.0/)

