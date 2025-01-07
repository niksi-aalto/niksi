# Niksi CLI docs

This guide covers using the `niksi` command line tool to build and push a devenv container to the GitHub container registry.

## Authentication 

1. Go to https://github.com/settings/tokens
2. Create a classic token with permissions `delete:packages, repo, write:packages`
3. Copy the token
4. Create a new file `.credentials` in the root of the course materials repo
5. Open the file and write on one line your GitHub username and the token separated by a semicolon
  * Example: `jturpela:ghp_32niVtAKMYIJ6dxXa5vcTLxpsit4R30zNW74`

## Building and pushing

To build the container, run `niksi build`.
This only generates the Docker image but does not push it, which is good for debugging.

To build and push the image, run `niksi build --push`.
