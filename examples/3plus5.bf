+++     add 3 to c0
> +++++ add 5 to c1
<       move back to c0

[       if c0 is 0 jump to close
  - >   decrement c0 and move to c1
  + <   increment c1 and move back to c0
]       if c0 is not 0 jump to open

>   move onto c1
add 48 to c1 to get it to print as 8
++++ ++++ 
++++ ++++ 
++++ ++++ 
++++ ++++ 
++++ ++++ 
++++ ++++ 
.   print c1
