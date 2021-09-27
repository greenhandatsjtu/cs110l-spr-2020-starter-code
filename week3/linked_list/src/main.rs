use linked_list::LinkedList;

pub mod linked_list;

fn main() {
    let mut list: LinkedList<u32> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    let mut list1 = list.clone();
    list1.pop_front();
    list1.push_front(99);
    println!("\nlist: {}\nlist1: {}", list, list1);
    println!("list=list!? {}", list == list1);
    let list2 = list1.clone();
    println!("list2: {}", list2);
    println!("list1=list2? {}", list1 == list2);

    for n in list {
        println!("{}", n);
    }
    for n in &list1 {
        print!("{} ", n * 2);
    }
    println!();
    println!("list1: {}", list1);
    // If you implement iterator trait:
    //for val in &list {
    //    println!("{}", val);
    //}
}
