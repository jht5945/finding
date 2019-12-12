# finding
finding - command line find tool.

#### Install

```shell
cargo install finding

cargo install --git https://github.com/jht5945/finding
```

#### Help

```
$ finding --help
Usage:
  finding [OPTIONS] [SEARCH TEXT]

finding - command line find tool.

Positional arguments:
  SEARCH TEXT           Search text

Optional arguments:
  -h,--help             Show this help message and exit
  -t,--target TARGET    Target, text, huge[file], default text
  -d,--dir DIR          Target directory, default current dir(.)
  --huge-file HUGE_FILE Huge file size, default 100M
  --large-text-file LARGE_TEXT_FILE
                        Large text file, default 10M
  -f,--file-ext FILE_EXT
                        File ext, default all
  -i,--ignore-case      Ignore case, default false
  --filter-large-line   Filter large line
  --large-line-size LARGE_LINE_SIZE
                        Large line, default 10KB
  --scan-dot-git        Scan dot git
  --skip-dot-dir        Skipt dot dir [Text Mode]
  --skip-link-dir       Skip link dir
  --filter-file-name FILTER_FILE_NAME
                        Filter file name [Text Mode]
  --filter-line-content FILTER_LINE_CONTENT
                        Filter line content [Text Mode]
  -v,--version          Print version
  --verbose             Verbose
```


