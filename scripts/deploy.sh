#!/usr/bin/env bash

rsync -avhP --stats --del dist/ hybridscan.app:hybridscan
