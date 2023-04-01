# Energy Efficient Sporadic Task Schedular
---

## Table of Contents
---

- Summary
- Technologies
- Setup
- References


## Summary
---

This is a sporadic task schedular based on the research paper “On-line scheduling of hard real-time tasks on variable voltage processor,”.
The paper is aimed at scheduling tasks with variable voltages to save on cpu power consumption for portable devices. What I implemented here
was that STS algorithm described in the paper. This algorithm uses a utilization ratio to allow or deny new processes that get released. Using the
max utilization ratio from all the processes we can use that ratio to create a voltage multiple for the cpu saving the need to run the process at full
power. In this instance of the algorithm I also included context time and I chanched the RS numerator to consider context switching time of the processes.
The schedular just prints out the results for now you can use standard output in the command line '>' to output the results to any desired file.

## Technologies
---
- Rust (please excuse the messy code just started learning it a week ago)


## Setup
---
1. locate the `sts_schedular` executable in `target/release/`
2. ```./sts_schedular -- <input_file.txt> > <output_file.txt>```
3. check results in the output file specified

## References
---

I. Hong, M. Potkonjak, and M. B. Srivastava, “On-line scheduling of hard real-time tasks on variable voltage processor,” Proc. Computer-Aided Design (ICCAD), pages 653–656, November 1998.