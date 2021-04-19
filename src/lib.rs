
// Lets use wee_alloc.
//#[cfg(feature = "wee_alloc")]
//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//
//#[wasm_bindgen]
//extern {
//    fn alert(s: &str);
//}
//
//#[wasm_bindgen]
//pub fn greet() {
//    alert("Hello, wasm-game-of-life!");
//}
//

////////////////////////////////////////////////////////////////////
///// Imports //////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////

use std::convert::TryFrom;
use std::cell::Cell;
use std::mem;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use rand::Rng;
use rand::distributions::weighted::alias_method::WeightedIndex;
use rand::distributions::Distribution;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
extern crate web_sys;
//extern crate getrandom;
//use lazy_static::lazy_static;
//use std::sync::Mutex;

////////////////////////////////////////////////////////////////////
///// Macros ///////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

////////////////////////////////////////////////////////////////////
// Custom data types. //////////////////////////////////////////////
////////////////////////////////////////////////////////////////////

struct TabUnits {
    units: Vec<TabUnit>,
}

impl TabUnits {
    fn add(&mut self, unit: TabUnit) {
        self.units.push(unit);
    }
    fn replace(&mut self, unit_id: u32, unit: TabUnit) {
        let x :usize = usize::try_from(unit_id).unwrap();
        mem::replace(&mut self.units[x], unit);
    }
}

impl fmt::Display for TabUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ id: {}, file_name: {}, unit_name: {}, faction: {}, cost: {}, enabled: {} }} :: TabUnit",
            self.id,
            self.file_name,
            self.unit_name,
            self.faction,
            self.cost,
            self.enabled)
    }
}

impl fmt::Display for TabUnits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        for i in self.units.iter() {
            result.push_str(&i.to_string());
            result.push_str("\n");
        }
        write!(f, "{}",result) 
    }
} 

static mut TABUNITS : TabUnits = TabUnits {
    units: Vec::new(),
};

static mut PRICE :u32 = 5000;

unsafe fn inc_price() -> u32 {
  if PRICE < 20000 { PRICE += 50; }
  PRICE
}

unsafe fn dec_price() -> u32 {
  if PRICE > 50 { PRICE -= 50; }
  PRICE
}

//lazy_static! {
//    static ref TABUNITS : Mutex<TabUnits> = Mutex::new(TabUnits {
//        units: Vec::new(),
//    });
//}

//fn tab_units_add(unit: TabUnit) {
//    TABUNITS.lock().unwrap().add(unit);
//}
//
//fn tab_units_replace(unit_id: u32, unit: TabUnit) {
//    TABUNITS.lock().unwrap().replace(unit_id, unit);
//}

#[wasm_bindgen]
#[derive(Clone)]
struct TabUnit {
   id: u32,  
   file_name: String,  
   unit_name: String,  
   faction: String, 
   cost: u32, 
   pub enabled: bool, 
}

unsafe fn select_all() {
    for i in TABUNITS.units.iter() {

        TABUNITS.replace(i.id, TabUnit {
                    id: i.id,
                    file_name: i.file_name.to_string(),
                    unit_name: i.unit_name.to_string(),
                    faction: i.faction.to_string(),
                    cost: i.cost,
                    enabled: true});
    }
}

unsafe fn select_clan(clan: String) {
    for i in TABUNITS.units.iter() {

        if i.faction == clan {
            //log!("juu loytyi id: {}, {}", i.id,  i.faction);
            TABUNITS.replace(i.id, TabUnit {
                        id: i.id,
                        file_name: i.file_name.to_string(),
                        unit_name: i.unit_name.to_string(),
                        faction: i.faction.to_string(),
                        cost: i.cost,
                        enabled: true});
        }
    }
}

unsafe fn disable_all() {
    for i in TABUNITS.units.iter() {

        TABUNITS.replace(i.id, TabUnit {
                    id: i.id,
                    file_name: i.file_name.to_string(),
                    unit_name: i.unit_name.to_string(),
                    faction: i.faction.to_string(),
                    cost: i.cost,
                    enabled: false});
    }
}

unsafe fn refresh_unit_elements() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    for i in TABUNITS.units.iter() {
        //let table = document.get_element_by_id("units_table").unwrap();
        let element = document.get_element_by_id(&format!("unit{}", i.id)).unwrap();      
        let mut enabled = String::new();
        if i.enabled == true { enabled.push_str("unit_enabled") }  
        else { enabled.push_str("unit_disabled") } 
        // log!("{}",format!("{} : {}", i.id, enabled));
        element.set_attribute("class", &format!("unit_div {}", enabled)).unwrap();
    }
}

unsafe fn collect_units(money: u32) -> HashMap<u32,u32> {

    let mut collection = Vec::new(); // = TABUNITS.units.clone(); // Vec::new();

    for i in TABUNITS.units.iter() {
        if i.enabled == true {
            collection.push(i.clone());
        }
    }


    collection.sort_by(|a,b| a.cost.partial_cmp(&b.cost).unwrap());
    let filtered_collection : Vec<TabUnit> = collection.into_iter().filter(|x| x.cost <= money).collect();

    let mut army_hash_map = HashMap::new();

    if filtered_collection.len() == 0 { return army_hash_map }


    let funcs: Vec<fn(TabUnit) -> u32> = vec![
            |x: TabUnit| if x.cost < 121 {98} else if x.cost < 501 {50} else {2},  
            |x: TabUnit| if x.cost < 81 {98} else if x.cost < 121 {75} else {5},  
            |x: TabUnit| if x.cost < 151 {95} else if x.cost < 251 {50} else {5},  
            |x: TabUnit| if x.cost < 151 {25} else if x.cost < 251 {35} else if x.cost < 510 {95} else {5},  
            |x: TabUnit| if x.cost < 990 {5} else if x.cost < 1995 {95} else {10},  
            |x: TabUnit| if x.cost < 251 {95} else if x.cost < 2995 {5} else {95},  
            |x: TabUnit| if x.cost < 301 {3} else if x.cost < 751 {95} else if x.cost < 1501 {3} else {10},  
            |x: TabUnit| if x.cost < 2999 {5} else {95}  
        ]; 

    let dist_type = rand::thread_rng().gen_range(0,funcs.len());

    let mut weigths: Vec<TabUnit> = filtered_collection.clone();  
    let mut weights2: Vec<u32> = weigths.into_iter().map(funcs[dist_type as usize]).collect::<Vec<u32>>();
    //let mut weights2: Vec<u32> = weigths.into_iter().map(|x| if x.cost < 250 { 80 }
    //                            else if x.cost < 500 {50 } 
    //                            else if x.cost < 1000 {33 }
    //                            else if x.cost < 1500 {25 }
    //                            else if x.cost < 2000 {10 }
    //                            else {5}
    //                            ).collect::<Vec<u32>>();

    //let dist = WeightedIndex::new(weights).unwrap();
    //for i in weights2.iter() { log!("{}", i) }

    let mut done = false;
    let mut money_left = money;
    let mut upper_index = filtered_collection.len() as u32; 
    let dist = WeightedIndex::new(weights2).unwrap();
    let choices: Vec<u32> = (0..upper_index).collect();
    let mut rng = rand::thread_rng();

    while !done {
        //let mut r = rand::thread_rng().gen_range(0,upper_index); 
        //let mut r = weights2.sample(&mut rng);rand::thread_rng().gen_range(0,upper_index); 
        let mut r = choices[dist.sample(&mut rng)];
        if money_left < 50 { break; } 
        let id = filtered_collection[r as usize].id;
        let price = filtered_collection[r as usize].cost;
        //log!("money_left {} < price {}", money_left, price);
        if money_left < price {
            //log!("Updating upper index. Old value {}. New value {}", upper_index, r);
            upper_index = r;
            if upper_index == 0 { break }
            else { continue }
            }

        //log!("Adding {} :: price {} in index {}. Upperindex = {}",TABUNITS.units[id as usize].unit_name,
        //                                                          TABUNITS.units[id as usize].cost,
        //                                                          r, upper_index );
        *army_hash_map.entry(id).or_insert(0) += 1;
        money_left -= price;
        //log!("Money left = {}",money_left);
    }

    let mut sum = 0;
    for (i,j) in army_hash_map.iter() {
      log!("{} :: price {} amount = {} total cost {}",TABUNITS.units[*i as usize].unit_name,TABUNITS.units[*i as usize].cost, *j, TABUNITS.units[*i as usize].cost * *j);
      sum += TABUNITS.units[*i as usize].cost * *j;
    }
    log!("Total sum {}.", sum);

    army_hash_map

}

////////////////////////////////////////////////////////////////////
// Functions ///////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {

    create_initial_units(); 
    create_initial_elements().unwrap();
    create_table()?;  

    Ok(())
}

//// TODO: Remove this.
//#[wasm_bindgen]
//pub fn add(a: u32, b: u32) -> u32 {
//    a + b
//}

pub fn create_initial_elements() -> Result<(), JsValue>  {

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    // let ul = document.create_element("ul")?;

    let main_container = document.create_element("div")?;
    main_container.set_attribute("id", "main_container").unwrap();

    // RIGHT CONTAINER
    let right_container = document.create_element("div")?;
    right_container.set_attribute("id", "right_container").unwrap();

    // BUTTONS CONTAINER
    let buttons_container = document.create_element("div")?;
    buttons_container.set_attribute("id", "buttons_container").unwrap();

    // ALL BUTTON
    let select_all_button = document.create_element("button")?.dyn_into::<web_sys::HtmlButtonElement>()?;
    let select_all_text = document.create_text_node("Enable all");
    select_all_button.append_child(&select_all_text)?;
    select_all_button.set_attribute("id", "select_all_button").unwrap();
    buttons_container.append_child(&select_all_button)?;


    // ALL BUTTON PRESSED.
    let pressed = Rc::new(Cell::new(false));
    {
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    pressed.set(true);
                    
                    unsafe { select_all() }
                    unsafe { refresh_unit_elements() }
                     
                    }) as Box<dyn FnMut(_)>);
        select_all_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
        closure.forget();
    }

    // RESET BUTTON
    let reset_button = document.create_element("button")?.dyn_into::<web_sys::HtmlButtonElement>()?;
    let reset_button_text = document.create_text_node("Disable all");
    reset_button.append_child(&reset_button_text)?;
    reset_button.set_attribute("id", "reset_button").unwrap();
    buttons_container.append_child(&reset_button)?;

    // DISABLE ALL BUTTON PRESSED.
    let pressed = Rc::new(Cell::new(false));
    {
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    pressed.set(true);
                    
                    unsafe { disable_all() }
                    unsafe { refresh_unit_elements() }
                     
                    }) as Box<dyn FnMut(_)>);
        reset_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
        closure.forget();
    }


    let clans = vec!["Tribal","Farmer","Medieval","Ancient","Viking","Dynasty","Renaissance","Pirate","Spooky","Western","Secret"];

    for i in clans.iter() {
      let clan_button = document.create_element("button")?;
      let clan_button_text = document.create_text_node(i);
      //let clan = i.to_string();

      // ALL BUTTON PRESSED.
      let pressed = Rc::new(Cell::new(false));
      {
          let pressed = pressed.clone();
          let c = i.clone();
          // log!("{}",c == true) ;
          let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                      pressed.set(true);
                      
                      unsafe { select_clan(c.to_string()) }
                      unsafe { refresh_unit_elements() }
                       
                      }) as Box<dyn FnMut(_)>);
          clan_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
          closure.forget();
      }

      clan_button.append_child(&clan_button_text)?;
      buttons_container.append_child(&clan_button)?;
    }
    let table_result_div = document.create_element("div")?;
    table_result_div.set_attribute("id", "div_army").unwrap();

    let table = document.create_element("table")?;
    let table_result = document.create_element("table")?;

    let thead = document.create_element("thead")?;
    let thead_result = document.create_element("thead")?;
    //let tr = document.create_element("tr")?;
    table.set_attribute("id", "units_table").unwrap();
    table_result.set_attribute("id", "army_table").unwrap();

    let tbody = document.create_element("tbody")?;
    let tbody_result = document.create_element("tbody")?;
    tbody.set_attribute("id", "units_table_body").unwrap();
    tbody_result.set_attribute("id", "army_table_body").unwrap();

    table_result_div.append_child(&table_result);
    
    // Do we really need headers for table?
//    for i in 0 .. 8 {
//        let th2 = document.create_element("th")?;
//        let text_th2 = document.create_text_node("Name");
//        th2.append_child(&text_th2)?;
//        th2.set_attribute("colspan", "1");
//
//        let th3 = document.create_element("th")?;
//        let text_th3 = document.create_text_node("Cost");
//        th2.append_child(&text_th3)?;
//        th2.set_attribute("colspan", "1");
//
//        tr.append_child(&th2)?;
//        tr.append_child(&th3)?;
//    }

    table.append_child(&tbody)?;
    table.append_child(&thead)?;
    table_result.append_child(&tbody_result)?;
    table_result.append_child(&thead_result)?;

    main_container.append_child(&table)?;
    main_container.append_child(&right_container)?;

    right_container.append_child(&buttons_container)?;
    right_container.append_child(&table_result_div)?;
    body.append_child(&main_container)?;

    // PRICE AND +/- BUTTONS

    let price_div = document.create_element("div")?;
    price_div.set_attribute("id", "price_div").unwrap();

    let price_label = document.create_element("label")?;

    price_label.set_attribute("id", "price_label").unwrap();
    price_label.set_attribute("maxlength", "6").unwrap();

    let mut i_price = 5000;
    unsafe { i_price =  PRICE }
    let initial_price = document.create_text_node(&format!("{}", i_price));
    //initial_price.set_attribute("id", "initial_price").unwrap();
    price_label.append_child(&initial_price)?;
    price_div.append_child(&price_label)?;


    let plus_minus_div = document.create_element("div")?;
    plus_minus_div.set_attribute("id", "plus_minus_div").unwrap();

    let plus_button = document.create_element("button")?.dyn_into::<web_sys::HtmlButtonElement>()?;
    let minus_button = document.create_element("button")?.dyn_into::<web_sys::HtmlButtonElement>()?;

    // PLUS BUTTON PRESSED.
    let pressed = Rc::new(Cell::new(false));
    {
        let w = web_sys::window().expect("no global `window` exists");
        let d = w.document().expect("should have a document on window");
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    pressed.set(true);
                    let p = d.get_element_by_id("price_label").unwrap();
                    let new_price; unsafe { new_price =  inc_price() } 
                    p.set_text_content(Some(&format!("{}",new_price)));
                    }) as Box<dyn FnMut(_)>);
        plus_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
        closure.forget();
    }

    // PLUS MINUS PRESSED.
    let pressed = Rc::new(Cell::new(false));
    {
        let w = web_sys::window().expect("no global `window` exists");
        let d = w.document().expect("should have a document on window");
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    pressed.set(true);
                    let p = d.get_element_by_id("price_label").unwrap();
                    let mut new_price; unsafe { new_price =  dec_price() } 
                    p.set_text_content(Some(&format!("{}",new_price)));
                    }) as Box<dyn FnMut(_)>);
        minus_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
        closure.forget();
    }

    plus_button.set_attribute("id", "plus_button").unwrap();
    minus_button.set_attribute("id", "minus_button").unwrap();

    let plus_text = document.create_text_node("+");
    let minus_text = document.create_text_node("-");

    plus_button.append_child(&plus_text)?;
    minus_button.append_child(&minus_text)?;

    plus_minus_div.append_child(&plus_button)?;
    plus_minus_div.append_child(&minus_button)?;

    price_div.append_child(&plus_minus_div)?;
    buttons_container.append_child(&price_div)?;

    let random_button = document.create_element("button")?.dyn_into::<web_sys::HtmlButtonElement>()?;
    random_button.set_attribute("id", "random_button").unwrap();
    random_button.set_text_content(Some(&format!("{}","Create Army")));

    // CREATE ARMY PRESSED.
    let pressed = Rc::new(Cell::new(false));
    {
        let w = web_sys::window().expect("no global `window` exists");
        let d = w.document().expect("should have a document on window");
        let pressed = pressed.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    pressed.set(true);
                    let money;
                    unsafe { money = PRICE; }
                    let random_army;
                    unsafe { random_army = collect_units(money) }
                    create_army_table(&random_army);
                    //for (i, amount) in random_army.iter() {
                    //  //if *i == true { money = 66 }
                    //  //unsafe { log!("{}",format!("{} : {}", TABUNITS.units[*i as usize], amount)) }
                    //}
                    //let p = d.get_element_by_id("price_label").unwrap();
                    //let mut new_price; unsafe { new_price =  dec_price() } 
                    //p.set_text_content(Some(&format!("{}",new_price)));
                    }) as Box<dyn FnMut(_)>);
        random_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
        closure.forget();
    }

    buttons_container.append_child(&random_button)?;

    Ok(())
}

pub fn create_army_table(army_map: &HashMap<u32,u32>) -> Result<(), JsValue>  {

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    //let table = document.get_element_by_id("units_table").unwrap();
    let tbody = document.get_element_by_id("army_table_body").unwrap();
    tbody.set_inner_html(&"".to_string());
    tbody.set_attribute("id", "army_table_body").unwrap();

    let mut v: Vec<_> = army_map.into_iter().collect();
    unsafe { v.sort_by(|x,y| TABUNITS.units[*x.0 as usize].cost.cmp(&TABUNITS.units[*y.0 as usize].cost)); }
    unsafe { v.sort_by(|x,y| TABUNITS.units[*x.0 as usize].faction.cmp(&TABUNITS.units[*y.0 as usize].faction)); }

    unsafe {

        let mut counter = 0;
        let mut tr2 = document.create_element("tr")?;

        for (i,c) in v.iter() {
            let td_unit = document.create_element("td")?;

            if counter % 7 == 0 && counter != 0 {
              tbody.append_child(&tr2)?;
              tr2 = document.create_element("tr")?;
            }

            // The unit DIV.
            let unit_div = document.create_element("div")?.dyn_into::<web_sys::HtmlDivElement>()?;
            //unit_div.set_attribute("class", &format!("{}","unit_div"));
            
            // The Image.
            let image = document.create_element("img")?;
            let mut tab_unit;
            unsafe { tab_unit = TABUNITS.units[**i as usize].clone() }
            image.set_attribute("alt", "").unwrap();
            image.set_attribute("width", "75").unwrap();
            image.set_attribute("height", "75").unwrap();
            image.set_attribute("src", &format!("{}{}","pics/",tab_unit.file_name).to_string()).unwrap();
            unit_div.append_child(&image)?;
            
            let mut enabled_text = String::new();
            // if i.enabled == true { enabled_text.push_str("unit_enabled") }
            // else { enabled_text.push_str("unit_enabled") } 

            unit_div.set_attribute("class", &format!("{}",&"unit_div_army")).unwrap();

            let div_cost = document.create_element("div")?;
            div_cost.set_attribute("class", &format!("{}","div_cost")).unwrap();
            let cost = document.create_text_node(&(c.to_string()));
            div_cost.append_child(&cost)?;

            unit_div.append_child(&image)?;
            unit_div.append_child(&div_cost)?;
            //unit_div.set_attribute("id", &format!("army{}", i.id)).unwrap();

            td_unit.append_child(&unit_div)?;
            tr2.append_child(&td_unit)?;

            //if TABUNITS.units.len() == counter + 1 {
            //  tbody.append_child(&tr2)?;
            //}

            counter = counter + 1;
        }
            tbody.append_child(&tr2)?;
    }

    Ok(())

}

// Update the units in table. Table and tbody for the table should exist before caling this
// function.
pub fn create_table() -> Result<(), JsValue>  {

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    //let table = document.get_element_by_id("units_table").unwrap();
    let tbody = document.get_element_by_id("units_table_body").unwrap();

    unsafe {

        let mut counter = 0;
        let mut tr2 = document.create_element("tr")?;

        for i in TABUNITS.units.iter() {
            let td_unit = document.create_element("td")?;

            if counter % 7 == 0 && counter != 0 {
              tbody.append_child(&tr2)?;
              tr2 = document.create_element("tr")?;
            }

            // The unit DIV.
            let unit_div = document.create_element("div")?.dyn_into::<web_sys::HtmlDivElement>()?;
            //unit_div.set_attribute("class", &format!("{}","unit_div"));
            
            // The Image.
            let image = document.create_element("img")?;
            image.set_attribute("alt", "").unwrap();
            image.set_attribute("width", "75").unwrap();
            image.set_attribute("height", "75").unwrap();
            image.set_attribute("src", &format!("{}{}","pics/",i.file_name).to_string()).unwrap();
            unit_div.append_child(&image)?;
            
            let mut enabled_text = String::new();
            if i.enabled == true { enabled_text.push_str("unit_enabled") }
            else { enabled_text.push_str("unit_enabled") } 

            unit_div.set_attribute("class", &format!("unit_div {}", enabled_text.to_string())).unwrap();

            let div_cost = document.create_element("div")?;
            div_cost.set_attribute("class", &format!("{}","div_cost")).unwrap();
            let cost = document.create_text_node(&(i.cost.to_string()));
            div_cost.append_child(&cost)?;

            unit_div.append_child(&image)?;
            unit_div.append_child(&div_cost)?;
            unit_div.set_attribute("id", &format!("unit{}", i.id)).unwrap();

            td_unit.append_child(&unit_div)?;
            tr2.append_child(&td_unit)?;

            // Mouse event.
            let pressed = Rc::new(Cell::new(false));
            //let t_units = Rc::new(Cell::new(TABUNITS));
            {
                let pressed = pressed.clone();
                //let item = Cell::new(i.clone());
                //let item = i.clone();
                let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                            pressed.set(true);

                            TABUNITS.replace(i.id, TabUnit {
                                                             id: i.id,
                                                             file_name: i.file_name.to_string(),
                                                             unit_name: i.unit_name.to_string(),
                                                             faction: i.faction.to_string(),
                                                             cost: i.cost,
                                                             enabled: !i.enabled});

                            // unsafe {log!("{}", TABUNITS)} 
                            let w = web_sys::window().expect("no global `window` exists");
                            let d = w.document().expect("should have a document on window");
                            let u = d.get_element_by_id(&format!("unit{}", i.id)).unwrap();
                            
                            //TODO: refactor this.
                            let mut e_text = String::new();
                            if i.enabled == true { e_text.push_str("unit_enabled") }
                            else { e_text.push_str("unit_disabled") } 
                            u.set_attribute("class", &format!("unit_div {}", e_text.to_string())).unwrap();
                             
                            }) as Box<dyn FnMut(_)>);
                unit_div.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?; 
                closure.forget();
            }

            // log!("{}", TABUNITS.units.len());
            if TABUNITS.units.len() == counter + 1 {
              tbody.append_child(&tr2)?;
            }

            counter = counter + 1;
        }
    }

    Ok(())

}

pub fn create_initial_units() {

    unsafe { TABUNITS.add(TabUnit {id: 0, file_name: "1.png".to_string(),  unit_name: "Clubber".to_string(),   faction: "Tribal".to_string(), cost: 70, enabled: true }); } 
    unsafe { TABUNITS.add(TabUnit {id: 1, file_name: "2.png".to_string(),  unit_name: "Protector".to_string(), faction: "Tribal".to_string(), cost: 90, enabled: true }); } 
    unsafe { TABUNITS.add(TabUnit {id: 2, file_name: "3.png".to_string(),  unit_name: "Spear Thrower".to_string(), faction: "Tribal".to_string(), cost: 120, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 3, file_name: "4.png".to_string(),  unit_name: "Stoner".to_string(),    faction: "Tribal".to_string(), cost: 160, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 4, file_name: "5.png".to_string(),  unit_name: "Bone Mage".to_string(), faction: "Tribal".to_string(), cost: 300, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 5, file_name: "6.png".to_string(),  unit_name: "Chieftain".to_string(), faction: "Tribal".to_string(), cost: 400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 6, file_name: "7.png".to_string(),  unit_name: "Mammoth".to_string(),   faction: "Tribal".to_string(), cost: 2200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 7, file_name: "8.png".to_string(),  unit_name: "Halfling".to_string(),  faction: "Farmer".to_string(), cost: 50, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 8, file_name: "9.png".to_string(),  unit_name: "Farmer".to_string(),  faction: "Farmer".to_string(), cost: 70, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 9, file_name: "10.png".to_string(), unit_name: "Hay baler".to_string(),  faction: "Farmer".to_string(), cost: 140, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 10, file_name: "11.png".to_string(), unit_name: "Potion seller".to_string(),  faction: "Farmer".to_string(), cost: 340, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 11, file_name: "12.png".to_string(), unit_name: "Harvester".to_string(),  faction: "Farmer".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 12, file_name: "13.png".to_string(), unit_name: "Wheelbarrow".to_string(),  faction: "Farmer".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 13, file_name: "14.png".to_string(), unit_name: "Scarecrow".to_string(),  faction: "Farmer".to_string(), cost: 1200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 14, file_name: "15.png".to_string(), unit_name: "Bard".to_string(),  faction: "Medieval".to_string(), cost: 60, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 15, file_name: "16.png".to_string(), unit_name: "Squire".to_string(),  faction: "Medieval".to_string(), cost: 100, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 16, file_name: "17.png".to_string(), unit_name: "Archer".to_string(),  faction: "Medieval".to_string(), cost: 140, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 17, file_name: "18.png".to_string(), unit_name: "Healer".to_string(),  faction: "Medieval".to_string(), cost: 180, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 18, file_name: "19.png".to_string(), unit_name: "Knight".to_string(),  faction: "Medieval".to_string(), cost: 650, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 19, file_name: "20.png".to_string(), unit_name: "Catapult".to_string(),  faction: "Medieval".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 20, file_name: "21.png".to_string(), unit_name: "The King".to_string(),  faction: "Medieval".to_string(), cost: 1500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 21, file_name: "22.png".to_string(), unit_name: "Shield bearer".to_string(),  faction: "Ancient".to_string(), cost: 100, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 22, file_name: "23.png".to_string(), unit_name: "Sarissa".to_string(),  faction: "Ancient".to_string(), cost: 120, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 23, file_name: "24.png".to_string(), unit_name: "Hoplite".to_string(),  faction: "Ancient".to_string(), cost: 180, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 24, file_name: "25.png".to_string(), unit_name: "Snake Archer".to_string(),  faction: "Ancient".to_string(), cost: 300, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 25, file_name: "26.png".to_string(), unit_name: "Ballista".to_string(),  faction: "Ancient".to_string(), cost: 900, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 26, file_name: "27.png".to_string(), unit_name: "Minotaur".to_string(),  faction: "Ancient".to_string(), cost: 1600, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 27, file_name: "28.png".to_string(), unit_name: "Zeus".to_string(),  faction: "Ancient".to_string(), cost: 2000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 28, file_name: "29.png".to_string(), unit_name: "Headbutter".to_string(),  faction: "Viking".to_string(), cost: 90, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 29, file_name: "30.png".to_string(), unit_name: "Ice Archer".to_string(),  faction: "Viking".to_string(), cost: 160, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 30, file_name: "31.png".to_string(), unit_name: "Brawler".to_string(),  faction: "Viking".to_string(), cost: 220, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 31, file_name: "32.png".to_string(), unit_name: "Berserker".to_string(),  faction: "Viking".to_string(), cost: 250, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 32, file_name: "33.png".to_string(), unit_name: "Valkyrie".to_string(),  faction: "Viking".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 33, file_name: "34.png".to_string(), unit_name: "Jarl".to_string(),  faction: "Viking".to_string(), cost: 850, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 34, file_name: "35.png".to_string(), unit_name: "Longship".to_string(),  faction: "Viking".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 35, file_name: "36.png".to_string(), unit_name: "Samurai".to_string(),  faction: "Dynasty".to_string(), cost: 140, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 36, file_name: "37.png".to_string(), unit_name: "Firework Archer".to_string(),  faction: "Dynasty".to_string(), cost: 180, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 37, file_name: "38.png".to_string(), unit_name: "Monk".to_string(),  faction: "Dynasty".to_string(), cost: 250, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 38, file_name: "39.png".to_string(), unit_name: "Ninja".to_string(),  faction: "Dynasty".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 39, file_name: "40.png".to_string(), unit_name: "Dragon".to_string(),  faction: "Dynasty".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 40, file_name: "41.png".to_string(), unit_name: "Hwacha".to_string(),  faction: "Dynasty".to_string(), cost: 1500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 41, file_name: "42.png".to_string(), unit_name: "Monkey King".to_string(),  faction: "Dynasty".to_string(), cost: 2000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 42, file_name: "61.png".to_string(), unit_name: "Painter".to_string(),  faction: "Renaissance".to_string(), cost: 50, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 43, file_name: "62.png".to_string(), unit_name: "Fencer".to_string(),  faction: "Renaissance".to_string(), cost: 150, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 44, file_name: "63.png".to_string(), unit_name: "Balloon Archer".to_string(),  faction: "Renaissance".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 45, file_name: "64.png".to_string(), unit_name: "Musketeer".to_string(),  faction: "Renaissance".to_string(), cost: 250, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 46, file_name: "65.png".to_string(), unit_name: "Halberd".to_string(),  faction: "Renaissance".to_string(), cost: 400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 47, file_name: "66.png".to_string(), unit_name: "Jouster".to_string(),  faction: "Renaissance".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 48, file_name: "67.png".to_string(), unit_name: "Da Vinci Tank".to_string(),  faction: "Renaissance".to_string(), cost: 4000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 49, file_name: "70.png".to_string(), unit_name: "Flintlock".to_string(),  faction: "Pirate".to_string(), cost: 100, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 50, file_name: "71.png".to_string(), unit_name: "Blunderbuss".to_string(),  faction: "Pirate".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 51, file_name: "72.png".to_string(), unit_name: "Bomb Thrower".to_string(),  faction: "Pirate".to_string(), cost: 250, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 52, file_name: "73.png".to_string(), unit_name: "Harpooner".to_string(),  faction: "Pirate".to_string(), cost: 400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 53, file_name: "74.png".to_string(), unit_name: "Cannon".to_string(),  faction: "Pirate".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 54, file_name: "75.png".to_string(), unit_name: "Captain".to_string(),  faction: "Pirate".to_string(), cost: 1500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 55, file_name: "76.png".to_string(), unit_name: "Pirate Queen".to_string(),  faction: "Pirate".to_string(), cost: 2500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 56, file_name: "84.png".to_string(), unit_name: "Skeleton Warrior".to_string(),  faction: "Spooky".to_string(), cost: 80, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 57, file_name: "85.png".to_string(), unit_name: "Skeleton Archer".to_string(),  faction: "Spooky".to_string(), cost: 180, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 58, file_name: "86.png".to_string(), unit_name: "Candlehead".to_string(),  faction: "Spooky".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 59, file_name: "87.png".to_string(), unit_name: "Vampire".to_string(),  faction: "Spooky".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 60, file_name: "88.png".to_string(), unit_name: "Pumpkin Catapult".to_string(),  faction: "Spooky".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 61, file_name: "89.png".to_string(), unit_name: "Swordcaster".to_string(),  faction: "Spooky".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 62, file_name: "90.png".to_string(), unit_name: "Reaper".to_string(),  faction: "Spooky".to_string(), cost: 2500, enabled: true}); } 

    unsafe { TABUNITS.add(TabUnit {id: 63, file_name: "101.png".to_string(), unit_name: "Dynamite Thrower".to_string(),  faction: "Western".to_string(), cost: 120, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 64, file_name: "92.png".to_string(), unit_name: "Miner".to_string(),  faction: "Western".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 65, file_name: "93.png".to_string(), unit_name: "Cactus".to_string(),  faction: "Western".to_string(), cost: 400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 66, file_name: "94.png".to_string(), unit_name: "Gunslinger".to_string(),  faction: "Western".to_string(), cost: 650, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 67, file_name: "95.png".to_string(), unit_name: "Lasso".to_string(),  faction: "Western".to_string(), cost: 850, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 68, file_name: "96.png".to_string(), unit_name: "Deadeye".to_string(),  faction: "Western".to_string(), cost: 900, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 69, file_name: "97.png".to_string(), unit_name: "Quick Draw".to_string(),  faction: "Western".to_string(), cost: 1200, enabled: true}); } 

    unsafe { TABUNITS.add(TabUnit {id: 70, file_name: "68.png".to_string(), unit_name: "Ballooner".to_string(),  faction: "Secret".to_string(), cost: 90, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 71, file_name: "77.png".to_string(), unit_name: "Bomb on a Stick".to_string(),  faction: "Secret".to_string(), cost: 140, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 72, file_name: "43.png".to_string(), unit_name: "Fan Bearer".to_string(),  faction: "Secret".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 73, file_name: "78.png".to_string(), unit_name: "Raptor".to_string(),  faction: "Secret".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 74, file_name: "45.png".to_string(), unit_name: "The Teacher".to_string(),  faction: "Secret".to_string(), cost: 200, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 75, file_name: "79.png".to_string(), unit_name: "Shouter".to_string(),  faction: "Secret".to_string(), cost: 250, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 76, file_name: "44.png".to_string(), unit_name: "Jester".to_string(),  faction: "Secret".to_string(), cost: 300, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 77, file_name: "46.png".to_string(), unit_name: "Chu Ko Nu".to_string(),  faction: "Secret".to_string(), cost: 350, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 78, file_name: "47.png".to_string(), unit_name: "Executioner".to_string(),  faction: "Secret".to_string(), cost: 350, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 79, file_name: "48.png".to_string(), unit_name: "Taekwondo".to_string(),  faction: "Secret".to_string(), cost: 400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 80, file_name: "80.png".to_string(), unit_name: "Raptor Rider".to_string(),  faction: "Secret".to_string(), cost: 480, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 81, file_name: "50.png".to_string(), unit_name: "Cheerleader".to_string(),  faction: "Secret".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 82, file_name: "91.png".to_string(), unit_name: "Cupid".to_string(),  faction: "Secret".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 83, file_name: "49.png".to_string(), unit_name: "Mace Spinner".to_string(),  faction: "Secret".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 84, file_name: "81.png".to_string(), unit_name: "CLAMS".to_string(),  faction: "Secret".to_string(), cost: 500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 85, file_name: "51.png".to_string(), unit_name: "Vlad".to_string(),  faction: "Secret".to_string(), cost: 1000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 86, file_name: "53.png".to_string(), unit_name: "Wheelbarrow Dragon".to_string(),  faction: "Secret".to_string(), cost: 1400, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 87, file_name: "82.png".to_string(), unit_name: "Bomb Cannon".to_string(),  faction: "Secret".to_string(), cost: 1500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 88, file_name: "54.png".to_string(), unit_name: "Cavalry".to_string(),  faction: "Secret".to_string(), cost: 1800, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 89, file_name: "69.png".to_string(), unit_name: "Lady Red Jade".to_string(),  faction: "Secret".to_string(), cost: 2000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 90, file_name: "83.png".to_string(), unit_name: "Blackbeard".to_string(),  faction: "Secret".to_string(), cost: 2500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 91, file_name: "55.png".to_string(), unit_name: "Shogun".to_string(),  faction: "Secret".to_string(), cost: 2800, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 92, file_name: "56.png".to_string(), unit_name: "Samurai Giant".to_string(),  faction: "Secret".to_string(), cost: 3000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 93, file_name: "57.png".to_string(), unit_name: "Sensei".to_string(),  faction: "Secret".to_string(), cost: 3000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 94, file_name: "52.png".to_string(), unit_name: "Ullr".to_string(),  faction: "Secret".to_string(), cost: 3000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 95, file_name: "58.png".to_string(), unit_name: "Tree Giant".to_string(),  faction: "Secret".to_string(), cost: 4000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 96, file_name: "59.png".to_string(), unit_name: "Ice Giant".to_string(),  faction: "Secret".to_string(), cost: 6000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 97, file_name: "60.png".to_string(), unit_name: "Artemis".to_string(),  faction: "Secret".to_string(), cost: 6500, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 98, file_name: "98.png".to_string(), unit_name: "Bank Robbers".to_string(),  faction: "Secret".to_string(), cost: 850, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 99, file_name: "99.png".to_string(), unit_name: "Gatling Gun".to_string(),  faction: "Secret".to_string(), cost: 2000, enabled: true}); } 
    unsafe { TABUNITS.add(TabUnit {id: 100, file_name: "100.png".to_string(), unit_name: "Ball n' Chain".to_string(),  faction: "Secret".to_string(), cost: 350, enabled: true}); } 
}
