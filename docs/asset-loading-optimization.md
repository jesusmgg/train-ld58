# Asset Loading Optimization

## Loading times baseline
Doing three runs for each test. All times in seconds.
Have in mind that in debug build and my test setup `mq_js_bundle.js` alone takes ~9.2s to load. we can disregard optimizing this.

### Sequential naive loads with separate assets
- 32.85
- 32.86
- 32.92

### Coroutine based texture loader
- 18.71
- 18.73
- 18.59

### Coroutine based texture and audio loaders
- 15.56
- 15.54
- 15.49
