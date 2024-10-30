# WorldGen 2

## Summary

This is a technical demo project of how to generate very large worlds using parallelisation (with rayon), and displaying them without running out of memory. Worldgen 2 uses an implementation of "scanline" method by using background rendering with threads and communicating with them, allowing parallel rendering and processing tensions in the world. The scanlines render chunks (set of tiles) n amount at a time, allowing the user to view the huge world. By default, 1 tile represents 1 tile, and the height map generated with summed Perlin noise components (essentially targeting a Brownian motion) is demonstrated by the darkness of a given pixel.
