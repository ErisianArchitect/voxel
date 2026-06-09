# voxel-orient

A library for handling all 48 voxel orientations without matrices or quaternions.

This library represents my research into performing voxel orientations as efficiently
as possible with as little memory as possible.

It is highly optimized. While performing batch operations, it's fast enough that you're
more likely to encounter memory bandwidth issues than you are to encounter performance
issues from my code.
