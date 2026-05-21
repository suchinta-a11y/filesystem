# How did you approach the assignment?
I started by first reading through the README file and try to get an understand of what the assingment is asking for. So my approach was taking it one step at a time. I started with defining the Node and filesystem types, and then moved on to handles and traits. 
For me persoanlly, the trickiest parts were the lifetimes in FileHandle and DirectoryHandle. It took me a little while to understand that the handle actually needs to borrow from the filesystem rather than copy the data. Finding the Find trait and implementing it for the FileHandle and DirectoryHandle proved to be another challenge for me, since I had to think carefully about how to share the recursive logic between the two implementations. 

# Who, if at all, did you work with?

I discussed the porject with Vishruth in the early-stage but the code and everything was my work. 

# What online resources helped when working on this lab?
- Rust reference on implementing Display and FromIterator
- Claude when I was really stuck or the instructions seemed cofnusing to me. 