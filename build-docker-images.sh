#!/bin/bash
# This script builds two docker images in a multi-stage build, i.e. 
    # 1. mc-crawler-builder - this produces an image with the compiled crawler
    # 2. mc-crawler - the crawler is copied from the above image into a minimal runtime environment so as to keep the image size as small as possible

docker build -t mc-crawler-builder -f Dockerfile.builder .
docker build -t mc-crawler -f Dockerfile.crawler .
