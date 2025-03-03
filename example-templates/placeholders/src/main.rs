fn main() {
    println!("Hello, {{authors}}!");
    println!("The boolean was: {{some_bool}}");
    println!("Your choice was: {{choice}}");
    println!("Your multi-choice was: {{multi_choice}}");

    {% if multi_choice contains "A" %}
        println!("You chose A!");
    {% endif %}
    {% if multi_choice contains "B" %}
        println!("You chose B!");
    {% endif %}
    {% if multi_choice contains "C" %}
        println!("You chose C!");
    {% endif %}
}
