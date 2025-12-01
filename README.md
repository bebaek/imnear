# Imnear

Command line app to search photos based on location metadata.

## Usage

```shell
Usage: imnear [OPTIONS] <RADIUS>

Arguments:
  <RADIUS>  Max distance from the target location

Options:
      --lat <LAT>                            Latitude of the target location
      --lon <LON>                            Longitude of the target location
      --address <ADDRESS>                    Address or search words
  -d, --dir <DIR>                            Directory/folder to search from [default: .]
  -e, --early-stop-count <EARLY_STOP_COUNT>  [default: -1]
  -s, --sort-by-distance
  -v, --verbose
  -h, --help                                 Print help
```
