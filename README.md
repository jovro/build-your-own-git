# Build Your Own Git

A WIP git implementation based on a [CodeCrafters challenge](https://app.codecrafters.io/courses/git/overview)

Only implement a small subset of git commands, cannot exactly be used as a replacement.
Mostly writes and reads data from the object storage, the interaction with branches, index and so forth is not there yet.
See the test instructions below, the results will be mostly found in the `.git/` directory.
The rust package manager [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) is required for the build.

How to test:

```sh
$ alias mygit=/path/to/repo/mygit.sh

$ mkdir test_dir && cd test_dir

# Init repo (Creates .git/)
$ mygit init

# Create file hashes
# -w flag will write the hash to .git/objects/3b/18e512dba79e4c8300dd08aeb37f8e728b8dad
# Other objects below will also be found at here
$ echo "hello world" > test.txt
$ mygit hash-object -w test.txt
3b18e512dba79e4c8300dd08aeb37f8e728b8dad

# Read the referenced file with the hash key
$ mygit cat-file 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
hello world

# Write tree to object storage
$ echo "hello world" > test_file_1.txt
$ mkdir test_dir_1
$ echo "hello world" > test_dir_1/test_file_2.txt
$ mkdir test_dir_2
$ echo "hello world" > test_dir_2/test_file_3.txt
$ mygit write-tree
26029824d815b92844579ec8a48000e496821bc1

# Read the previous file tree
$ mygit ls-tree 26029824d815b92844579ec8a48000e496821bc1

# Create commit from the write tree and store to object storage
$ mygit commit-tree 26029824d815b92844579ec8a48000e496821bc1 -m "Initial commit"
<SHA depends on timestamp>
```
