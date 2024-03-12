# rp Design Doc


## Stages

The plan is to start out rp as a single file prototype, and then, once the basics are in place, start expanding it.


### Architechture

The interpter is planned to be a stack-supplied state machine, such that all of the input states are stored on the interpreter struct. This is so that key-by-key commands can be handled directly by the interpreter, at least at first. (If this proves to be -too- complicated to implement, an input editor might be investigated, but I feel confident so far)

Eventually, there will be a TUI display. This I hope to write as a function of an immutable borrow of the stack machine.


### Mockups


Main screen, shown all of the time
```
/---------------------------\
|  0.#: 123                 |
|  1.#: 456                 |
| >2.#: 789_                |
|  3.$: [This is a string]  |
|                           |
| C-h: Help                 |
| C-r: Registers            |
| _: Change Sign            |
\---------------------------/
```


Registers screen, shown in slot two if enabled

```
/--------------------------\
| Registers:               |
| a.@: {1,[abc], 2}        |
| b.#: 3                   |
| c.$: [This is a string]  |
| i.$: [1+]                |
| g.F: f(x)=[x 1 +]        |
|                          |
\--------------------------/
```

Help screen, shown below any other screens, if enabled

```
/--------------------------\
| Help:                    |
| 1: Arithmetic            |
| 2: Trig                  |
| 3: Graphing              |
| 4: Search                |
| q: Exit Help             |
|                          |
\--------------------------/

```






















