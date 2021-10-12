#MySQL tricks


## Enums
To deserialize enums from a column into a struct field some boilerplate code is required. 
Fortunately there is a derive that does it. However to do the actual conversion between 
an enum and a string some other derive is required. There are many crates for this. 
Here is an example using strum.

cargo
[dependencies ]
strum =""
mysqlenum = ""
 
 #derive( Mysqlenum) 
 enum Mood {
    Happy,
    Sad
} 

Now _Mood_ can be used for Toql.

#[derive (Debug, Toql)]
struct User {
    id : u64,
    name: Option<string>
    mood: Option<Mood>
}


