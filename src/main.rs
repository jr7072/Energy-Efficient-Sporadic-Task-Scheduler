use std::fs;
use std::env;
use std::io;
use std::io::BufRead;
use std::cmp::Reverse;
use priority_queue::PriorityQueue;

#[derive(Hash, Eq)]
struct Process{
    task_id: String,
    computation: u16,
    deadline: u16,
    context_time: u8,
}


impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}


struct SchedularMeta{
    last_task: Option<String>, 
    last_context: Option<u8>,
    last_computation: Option<u16>,
    context: u8,
    current_voltage: f32,
}


fn split(input: &String, delimeter: char,
                    fixed: bool) -> (Vec<u16>, Vec<f32>){
    /*
        Splits the string representation of the process arguments.

        input: The arguments of the process in the form of [a, c, d, cs]

        delimeter: The character to split on

        returns a vector of integer representations for each argument.
    */

    let mut split_args: Vec<u16> = Vec::new();
    let mut split_speeds: Vec<f32> = Vec::new();

    let mut number: String = "".to_string();

    for character in input.chars(){

        if character == delimeter{
            
            if fixed{

                // cleaning up commas
                let number_len: usize = number.len();
                let number: String = number[..number_len - 1].
                                            to_string();

                let numeric: f32 = number.parse::<f32>().unwrap();
                split_speeds.push(numeric);
            }

            if !fixed{
                let numeric: u16 = number.parse::<u16>().unwrap();
                split_args.push(numeric);
            }

            number = "".to_string();
            
            continue;
        }

        number.push(character);
    }

    if fixed{
        let numeric: f32 = number.parse::<f32>().unwrap();
        split_speeds.push(numeric);
    }

    if !fixed{
        let numeric: u16 = number.parse::<u16>().unwrap();
        split_args.push(numeric);
    }

    return (split_args, split_speeds);

}


fn load_processes(filename: &String) -> (PriorityQueue<Process, Reverse<u16>>,
                                            Option<Vec<f32>>) {

    /*
        This function loads the process information from an input file
        into the program (My schedular will still assume that the tasks
        are dynamic).

        filename: The string representation of the filepath.

        returns: PriorityQueue where loaded processes are loaded
        in a priority queue
    */

    let file: fs::File = fs::File::open(filename).
                                        expect("error opening file.");
    
    let reader: io::BufReader<fs::File> = io::BufReader::new(file);

    let mut arrival_queue: PriorityQueue<Process, Reverse<u16>> = PriorityQueue::new();
    let mut speeds: Option<Vec<f32>> = None;

    for (index, line) in reader.lines().enumerate(){
        
        // don't need this information
        if index == 0{
            continue;
        }

        let line: String = line.unwrap().trim().to_string();
        let colon_position: usize = line.find(':').expect(": not present");

        let argument: &String = &line[0..colon_position].to_string();
        let process_arg_start: usize = colon_position + 3;
        let process_arg_end: usize = line.len() - 1;
        let process_args: &String = &line[process_arg_start..process_arg_end].
                                        to_string();

        // check if this is fixed case
        if *argument == "Possible speeds".to_string(){
            
            // remove the last )
            let process_args_len: usize = process_args.len();
            let speed_args: &String = &process_args[..process_args_len - 1].
                                        to_string();

            speeds = Some(split(speed_args, ' ', true).1);

            continue;
        }
        
        let arg_list: Vec<u16> = split(process_args, ' ', false).0;
        
        let arrival_time: u16 = arg_list[0];
        let computation_time: u16 = arg_list[1];
        let deadline: u16 = arg_list[2];
        let context_time: u8 = u8::try_from(arg_list[3]).
                                    expect("couldn't case u16 to u8");

        let process: Process = Process{
            task_id: argument.to_string(),
            computation: computation_time,
            deadline: deadline,
            context_time: context_time
        };

        arrival_queue.push(process, Reverse(arrival_time));
    }

    return (arrival_queue, speeds);

}


fn utilization(edf_queue: &PriorityQueue<Process, Reverse<u16>>, 
                        new_task: &Process, time: &f32,
                        speeds: &Option<Vec<f32>>) -> f32{
    /*
        Function to calulate utilization formula. RC is calculated
        using c_i + (2 * context)

        edf_queue: The current schedular queue being used to schedule
        tasks to the processor

        new_task: The proposed task to be added to the queue.

        returns: U(t) utilization ratio of all processes to including
        the new process
    */                            

    // calculate compuration sum
    let new_task_computation: f32 = f32::from(new_task.computation);
    let new_task_cs: f32 = f32::from(new_task.context_time);

    let mut computation_sum: f32 = new_task_computation + (2.0 * new_task_cs);
                    
    for item in edf_queue{
        
        let process: &Process = item.0;
        let process_computation: f32 = f32::from(process.computation);
        let process_cs: f32 = f32::from(process.context_time);

        let process_computation: f32 = process_computation + (2.0 * process_cs);
        computation_sum += process_computation;
    }

    let new_task_deadline: f32 = f32::from(new_task.deadline);

    //implicit assumption of max u_j being from the new task
    let mut max_uj = computation_sum / (new_task_deadline - time);

    for item in edf_queue{

        let process: &Process = item.0;
        let process_deadline: f32 = f32::from(process.deadline);

        let u_j: f32 = computation_sum / (process_deadline - time);

        if u_j > max_uj{
            max_uj = u_j;
        }

    }

    // fixed speed utilization
    if !speeds.is_none(){

        let speed_values: Option<Vec<f32>> = speeds.clone();

        for speed in speed_values.unwrap(){

            if speed > max_uj{
                max_uj = speed;
                return max_uj;
            }

        }

    }

    return max_uj;

}


fn queue_tasks(arrival_queue: &mut PriorityQueue<Process, Reverse<u16>>,
                time: &u16,
                current_voltage: &mut f32,
                edf_queue: &mut PriorityQueue<Process, Reverse<u16>>,
                speeds: &Option<Vec<f32>>){
    /*
        This function looks at any arriving tasks and adds them to the
        edf queue if they pass the utilization test. current voltage will
        be modified by reference as well.

        arrival_queue: The arrival queue where all tasks are stored.
        time: The current time unit
        current_voltage: The current voltage the cpu is on
        edf_queue: reference to the edf queue
    */
                    
 
    loop {
        
        if arrival_queue.is_empty(){
            return
        }
    
        // peek time
        let arrival_time: u16 = arrival_queue.peek().unwrap().1.0;

        if arrival_time != *time{
            break;
        }
        
        let arrived_process: Process = arrival_queue.pop().unwrap().0;

        let utilization_ratio: f32 = utilization(edf_queue,
                                            &arrived_process,
                                            &f32::from(*time),
                                            speeds);
        
        if utilization_ratio <= 1.0{
    
            // assign new voltage
            *current_voltage = utilization_ratio;

            
            let arrived_deadline: u16 = arrived_process.deadline;
            let absolute_deadline: u16 = arrival_time + arrived_deadline;
            
            edf_queue.push(arrived_process,
                                Reverse(absolute_deadline));
        } 

    }

}


fn process_task(edf_queue: &mut PriorityQueue<Process, Reverse<u16>>,
                    current_voltage: &f32,
                    last_computation: &mut Option<u16>,
                    context: &mut u8,
                    time: &u16){
    /*
        process the task that is recommended by the schedular with the
        appropriate voltage.

        edf_queue: The queue of processes calculated by the schedular.
        
        current_voltage: The voltage to run the processor at.
    */

    if *context > 0{
        
        println!("Time {}: Context", time);
        *context -= 1;
        return;
    }

    if edf_queue.is_empty(){
        println!("Time {}: No Process", time);
        return;
    }

    let next_item= edf_queue.pop().unwrap();
    let next_process: Process = next_item.0;
    let priority: Reverse<u16> = next_item.1;
    let task_id: &String = &next_process.task_id;

    println!("Time {}: Running {} at voltage {}", time, task_id, current_voltage);

    // update the task and last computation metadata  
    let new_computation_time:u16 = next_process.computation - 1;
    *last_computation = Some(new_computation_time);

    if new_computation_time == 0{
        // add context time at end
        let context_time: u8 = next_process.context_time;
        *context += context_time;
        return
    }
  

    let process_clone: Process = Process{
        task_id: task_id.clone(),
        computation: new_computation_time,
        deadline: next_process.deadline,
        context_time: next_process.context_time
    };

    edf_queue.push(process_clone, priority);

}


fn context_handler(metadata: &mut SchedularMeta,
                        edf_queue: &PriorityQueue<Process, Reverse<u16>>) {
    /*
        Handles context time calculation for the schedular

        metadata: The metadata struct that contains information relavent
        to the schedular

        edf_queue: The current edf queue generated by the schedular
    */


    if edf_queue.is_empty(){
        return;
    }

    let process: &Process = edf_queue.peek().unwrap().0;
    let task_id: String = process.task_id.clone();
    let process_context: u8 = process.context_time;

    if metadata.last_task == None{
        // case 0
        // scheduling first process
        metadata.last_task = Some(task_id.clone());
        metadata.last_context = Some(process_context);
        metadata.last_computation = Some(process.computation);
        metadata.context = process_context;

        return;
    };


    if metadata.last_task != Some(task_id.clone()){
        // case 1 and 2
        // Processes ended or preempted

        // task was preempted
        if metadata.last_computation > Some(0){

  
            // supply changes that were not caught by processor
            metadata.context += metadata.last_context.unwrap() + process_context;
            
            // update metadata
            metadata.last_task = Some(task_id.clone()); 
            metadata.last_computation = Some(process.computation);
            metadata.last_context = Some(process_context);

            return;
        }

        // previous task was completed just add the new process context
        if metadata.last_computation == Some(0){
            
            metadata.context += process_context;

            metadata.last_task = Some(task_id.clone()); 
            metadata.last_computation = Some(process.computation);
            metadata.last_context = Some(process_context);

            return
        }
    }

}


fn start_schedular(arrival_queue: &mut PriorityQueue<Process, Reverse<u16>>,
                        schedule_length: u16,
                        speeds: Option<Vec<f32>>){
    /*
        This is the sts schedular based on the paper referenced in assignment
        2. Processes are assumed to arrive dynamically even though processes
        from the input file are ariiving dynamically.

        The schedular operates in three stages
        1. queue tasks
        2. context handler
        2. process task

        input: arrival_queue a queue of processes from the input file
        sorted by thier arrival time.
    */

    let mut edf_queue: PriorityQueue<Process, Reverse<u16>> = PriorityQueue::new();

    let mut metadata: SchedularMeta = SchedularMeta{
        last_task: None,
        last_computation: None, 
        last_context: None,
        context: 0,
        current_voltage: 1.0 // start with the largest voltage
    };


    for time in 0..schedule_length{

        queue_tasks(arrival_queue, &time, &mut metadata.current_voltage,
                        &mut edf_queue, &speeds);
        
        context_handler(&mut metadata, &edf_queue);

        process_task(&mut edf_queue, &metadata.current_voltage,
                        &mut metadata.last_computation,
                        &mut metadata.context,
                        &time);
    }
    
}


fn main(){

    let arguments: Vec<String> = env::args().collect();
    let filename: &String = &arguments[1];
    let schedule_length: u16 = 200;

    let file_contents: (PriorityQueue<Process, Reverse<u16>>,
                            Option<Vec<f32>>) = load_processes(filename);
    let mut arrival_queue: PriorityQueue<Process, Reverse<u16>> = file_contents.0;
    let speeds: Option<Vec<f32>> = file_contents.1;

    start_schedular(&mut arrival_queue, schedule_length, speeds);

}