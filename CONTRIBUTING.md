# Contributing 
[![Good-first-Issue](https://img.shields.io/github/issues/yslide/slide/good%20first%20issue?style=flat-square)](https://github.com/yslide/slide/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) [![Github help-wanted](https://img.shields.io/github/issues/yslide/slide/help%20wanted?style=flat-square)](https://github.com/yslide/slide/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)

<br/>

Contributions of all kinds to slide are warmly welcomed. Here's an overview of the code base of a slide and how
to contribute.

## Building
slide is easy to build. To setup a simple environment do the following:

```
git clone {insert slide repo here}
cargo build
ladder test
# do your development
git commit -m {insert commit message}
git push 
```

This should allow you to create a fork of slide and push to your own github account! From here, you
can create a pull request with slide's development branch to make your awesome changes available to
all slide users!

## Documentation

Development documentation for slide, including documentation of private items, can be found in a
rustdoc at [slide-dev.ayazhafiz.com](https://slide-dev.ayazhafiz.com/libslide).

## Issues
Issues are filed similar to any open source project. Create a new issue with the proposed
feature or bug, add labels and submit. If the issue is a bug make sure you add the exact steps
required to reproduce the bug so that it can be solved efficiently. Also, to showcase a broken
expression, you can use [slide bot](#slide-bot) (see below). Filing issues are certainly an underrated part of any open source project and we appreciate all issues of any magnitude!

## Testing 
Generally, each change to slide should include tests. See [slide tests](./slide/src/test/README.md) for testing instructions.
### Ladder
Slide has its own development tool called [ladder](slide/src/test) that makes it easy to test for both errors and proper simplifications. It automatically
tests all "cargo test" unit tests implemented. The entire test system is described under slide/src/test. Basically, one just
has to create a file with their issue number and add their expected input/output in the same file using delimeters. Please 
take a look at the previous files for more clarification. Ladder is normally used for unit tests. If
you are writing system tests, check out the system test documentation. (needs to be written for
link)

### Slide-bot
Slide-bot allows you to test slide commands in git issues! It's simple, just open up your issue and
use `/slide "{insert issue}"` and slide bot will respond to the issue with slides exact output!
For example
```
/slide "1+2" 
```
Slide bot would then comment `3` on your issue!

## Where do I start?
Take a look at the slide [design doc](docs/DESIGN.md) for a general overview of slides architecture.
Every contribution no matter how little matters. With that said, refer to the issues marked as good
[first issue](https://github.com/yslide/slide/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) or 
[help wanted](https://github.com/yslide/slide/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%2)
to begin contributing to slide.
