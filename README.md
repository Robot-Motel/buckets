[![Rust](https://github.com/3vilM33pl3/buckets/actions/workflows/ci.yml/badge.svg)](https://github.com/3vilM33pl3/buckets/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)


!! Work in progress, only a few commands are implemented !!

### Overview
Buckets is a tool for game asset and expectation management. It controls the version of work a user creates and
sets and records expectations they have of each others work to improve collaboration. Every stage of the workflow is 
represented by a bucket which contains all the resources to create a game asset in a specific stage of the
production pipeline. The workflow is represented by linking buckets together by expectations, so that the output of 
one bucket is the input of another bucket.

Expectations are simple rules to indicate what a bucket needs to finalize it's work.
Once you finish your work in a bucket you can finalize it, which will automatically check all expectations and
when satisfied move the expected output of a bucket to the next bucket in the workflow.

### Example
Let's say you want to create a 3D model for a game. The model needs concept art and textures.
For this you can create two buckets with expectations. The first bucket contains concept art and the second bucket
contains the 3D model. The first bucket has three expectations:
1. The bucket has a mood board
2. The bucket has concept art
3. The concept art is approved by the art director

The second bucket has three expectations:
1. There is concept art for the model
2. The bucket has a 3D model
3. There are textures for the model

Both buckets are linked together so that the output of the first bucket is the input of the second bucket.

Once the concept art is ready and approved, you can 'finalize' the bucket. The second bucket will automatically receive the latest version of the concept art,
which will satisfy the first expectation. Now you can create the 3D model and textures. Once finished and all expectations are met
you can finalize the bucket, and it's ready for use in the game.

Buckets are generally defined per person or team who create a specific type of content. So if you are a 3D artist you will have a bucket for
your 3D models and textures and if you are a concept artist you will have a bucket for your concept art.

To make it possible to iterate over multiple versions you can give a version number to a finalized bucket.
Meaning you can have multiple 'final' versions of you assets. This is useful if you want to keep track of the
changes you made to an asset which have dependencies on other assets. For example, if you change the concept art
of a character, you will also have to change the 3D model and textures. By giving a version number you will know
which version of the 3D model and textures are based on which version of the concept art.

### Commands
`bucket init`
Initialize bucket repository

#### Buckets
`bucket create [name]`
Create a bucket for content

`bucket commit [message]`
Set the version of a bucket and store its content

`bucket finalize [version]`
Finalize a bucket and store its content

`bucket list`
Lists all buckets in a repository

`bucket history`
List all commits in a bucket

`bucket status`
Show which files have changed since the last commit

`bucket revert all`
Discards all changes and restores last commit

`bucket revert [file]`
Discards changes of a specific file and restores the file as it was in the
last commit

`bucket rollback [file] [commit id]`
Replaces a committed file in the bucket to the version found in the bucket with the specified commit id

`bucket rollback all [commit id]`
Replaces all committed files in the bucket with the versions found in the bucket with the specified commit id

`bucket stash`
Temporarily stashes the current version so you can retrieve another version

`bucket stash restore`
Restores stash

#### Rules and expectations
`bucket expect bucket [name]`
Expect the existence of a bucket with specified name

`bucket expect set file [type] [bucket directory]`
Set what file to expect in bucket

`bucket check`
Check if all expectations are met. If not, print what is missing.

`bucket link [from bucket directory] [to bucket directory]`
Create a one way link between two buckets

### Development Setup

When using VSCode for development:

1. Copy the VSCode settings template to create your local settings:
   ```bash
   cp .vscode/settings.json.template .vscode/settings.json
   ```

2. Adjust the rust-analyzer.rustc settings in `.vscode/settings.json` to match your local toolchain.

## License

This work is dual-licensed under Apache 2.0 and MIT.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR MIT`
