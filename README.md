# gmux

Manage multiple git repositories with ease.

`gmux` allows you to work on multiple, predefined, Git managed repositories.

## Pools

A pool is a collection of repositories and files.

Pools are at the heart of `gmux`: they allow you to easily target or exclude
repositories from your Git commands, but you can also share them easily as they
are simple YAML files.

## Commands

Building on the pool concept, you have two main commands using `gmux`.

By using `gmux pool`, you can manage your pools ; add new pools, automatically
populate them, clone repositories by using a pool, etc...

However, you can also forward any Git command by using `gmux command` and its
powerful `--filter` option.

## Use cases

`gmux` shines best when used by multiple team members working on a large set
of repositories.

Whereas it's to bootstrap the cloning process, or to ensure multiple
repositories track the same branches, you can share `gmux` configurations
to ensure everyone is on the same page.

However, you can also use it as a simple tool to query or edit multiple
repositories using Git commands.

## Examples

### Pools

Print the current pool to the console:

`gmux pool`

List all available pools:

`gmux pool`

Create a new pool, starting from the current directory:

`gmux pool new projects`

Create a new pool in a specific directory:

`gmux pool new python /home/user/projects/python`

Discover all repositories and files from the current pool directory:

`gmux pool discover`

Clone, checkout branches and recreate files of the current pool:

`gmux pool discover`

### Commands

Print the pool repositories status:

`gmux command status --short`

Print the pool repositories active branch that are not master or develop:

`gmux command --exclude-filter '(master|develop)' rev-parse --abbrev-ref HEAD`

Print the pool repositories ahead/behind commit count if there is any:

`gmux command --exclude-filter '0\s0' rev-list --left-right --count @...@{u}`

Print the pool repositories commits to merge with commiter and time, from oldest to newest:

`gmux command log --pretty=format:'%h%x09%cr%x09%cn%x09%s' --reverse @..@{u}`
