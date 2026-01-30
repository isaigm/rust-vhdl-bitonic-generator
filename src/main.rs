use rand::Rng;
use std::fs::File;
use std::io::{Write, BufWriter, Result};
use indoc::indoc;

struct VhdlWriter<'a> {
    id: usize,
    n: usize,
    data_width: usize,
    wires: Vec<String>,
    inputs: Vec<String>,
    writer: BufWriter<&'a File>, 
}

impl<'a> VhdlWriter<'a> {
    fn new(n: usize, data_width: usize, file: &'a File) -> Self {
        let total_comparators = ((n as f32).log2() * ((n as f32).log2() + 1.0) * (n as f32) / 4.0) as i32;
        
        let mut wires: Vec<String> = vec![];
        for i in 0..total_comparators {
            wires.push(format!("w_l_{i}"));
            wires.push(format!("w_h_{i}"));
        }

        let mut inputs: Vec<String> = vec![];
        for i in 0..n {
            inputs.push(format!("inputs({})", i));
        }

        VhdlWriter {
            n,
            data_width,
            id: 0,
            wires,
            inputs,
            writer: BufWriter::new(file),
        }
    }

    fn add_comparator(&mut self, dir: u32, in_a: &str, in_b: &str, out_l: &str, out_h: &str) -> Result<()> {
        writeln!(
            self.writer,
            "    comp_{id}: entity work.comparator port map(in_A => {in_a}, in_B => {in_b}, dir => '{dir}', out_L => {out_l}, out_H => {out_h});",
            id = self.id, in_a = in_a, in_b = in_b, dir = dir, out_l = out_l, out_h = out_h
        )?;
        Ok(())
    }

    fn write_wires(&mut self) -> Result<()> {
        for w in &self.wires {
            writeln!(self.writer, "    signal {} : std_logic_vector(width - 1 downto 0);", w)?;
        }
        writeln!(self.writer, "begin")?;
        Ok(())
    }

    fn write_start(&mut self) -> Result<()> {
        let header = format!(
            indoc! {"
                library IEEE;
                use IEEE.STD_LOGIC_1164.ALL;
                use IEEE.NUMERIC_STD.ALL;
                use work.network_types.all;

                entity bitonic_network is
                    generic (
                        n : integer := {n};
                        width : integer := {w}
                    );
                    Port (
                        inputs  : in mem(0 to n - 1)(width - 1 downto 0); 
                        outputs : out  mem(0 to n - 1)(width - 1 downto 0)       
                    );
                end bitonic_network;

                architecture Behavioral of bitonic_network is
                    signal outputs_array : mem(0 to n - 1)(width - 1 downto 0) := (others => (others => '0'));
            "},
            n = self.n,
            w = self.data_width
        );

        self.writer.write_all(header.as_bytes())?;
        Ok(())
    }

    fn write_end(&mut self) -> Result<()> {
        for (i, output) in self.inputs.iter().enumerate() {
            writeln!(self.writer,"    outputs_array({}) <= {};", i, output)?;
        }
        let end = indoc! {b"
                outputs <= outputs_array;
            end Behavioral;
        "};
        self.writer.write_all(end)?;
        Ok(())
    }
}

fn bitonic_merge(vhdl_writer: &mut VhdlWriter, count: usize, low: usize, dir: u32) -> Result<()> {
    if count > 1 {
        let k = count / 2;
        for i in low..low + k {
            let out_l = vhdl_writer.wires[vhdl_writer.id * 2].clone();
            let out_h = vhdl_writer.wires[vhdl_writer.id * 2 + 1].clone();
            let in_a = vhdl_writer.inputs[i].clone();
            let in_b = vhdl_writer.inputs[i + k].clone();

            vhdl_writer.add_comparator(dir, &in_a, &in_b, &out_l, &out_h)?;

            vhdl_writer.inputs[i] = vhdl_writer.wires[vhdl_writer.id * 2].clone();
            vhdl_writer.inputs[i + k] = vhdl_writer.wires[vhdl_writer.id * 2 + 1].clone();
            
            vhdl_writer.id += 1;
        }
        bitonic_merge(vhdl_writer, k, low, dir)?;
        bitonic_merge(vhdl_writer, k, low + k, dir)?;
    }
    Ok(())
}

fn bitonic_sort_helper(vhdl_writer: &mut VhdlWriter, count: usize, low: usize, dir: u32) -> Result<()> {
    if count > 1 {
        let k = count / 2;
        bitonic_sort_helper(vhdl_writer, k, low, 1)?;
        bitonic_sort_helper(vhdl_writer, k, low + k, 0)?;
        bitonic_merge(vhdl_writer, count, low, dir)?;
    }
    Ok(())
}

fn generate_sorter_file(n: usize, width: usize) -> Result<()> {
    let mut file = File::create("vhdl/sorter.vhd")?;
    let mut rng = rand::rng();
    
    let mut random_numbers_vhdl = Vec::new();
    let max_val = (1u32 << width) - 1;

    println!("--- Generated numbers ---");
    for _ in 0..n {
        let num = rng.random_range(0..=max_val);
        print!("{} ", num); 
        random_numbers_vhdl.push(format!("std_logic_vector(to_unsigned({}, {}))", num, width));
    }
    println!("\n-----------------------------------");

    let inputs_init_string = random_numbers_vhdl.join(",\n        ");

    let content = format!(
        indoc! {r#"
            library IEEE;
            use IEEE.STD_LOGIC_1164.ALL;
            use work.network_types.all;
            use IEEE.numeric_std.all;

            entity sorter is
                Port (
                    CLK100MHZ: in std_logic;
                    btnR: in std_logic;
                    LED: out std_logic_vector(15 downto 0)
                 );
            end sorter;

            architecture Behavioral of sorter is
                
                constant N_SYSTEM : integer := {n};
                constant W_SYSTEM : integer := {w};

                signal inputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (
                    {inputs_str}
                );
                
                signal outputs : mem(0 to N_SYSTEM - 1)(W_SYSTEM - 1 downto 0) := (others => (others => '0'));
                signal enabled: std_logic := '0';
                signal idx: integer range 0 to N_SYSTEM - 1 := 0;
                signal led_reg: std_logic_vector(15 downto 0) := (others => '0');

            begin
                network: entity work.bitonic_network 
                generic map (
                    n => N_SYSTEM,
                    width => W_SYSTEM
                )
                port map(
                    inputs => inputs, 
                    outputs => outputs
                );

                push_btn: entity work.push_btn port map(CLk100MHZ => CLK100MHZ, btn => btnR, enabled => enabled);

                process (CLK100MHZ)
                begin
                    if rising_edge(CLK100MHZ) then
                        if enabled = '1' then
                          
                            led_reg <= std_logic_vector(resize(unsigned(outputs(idx)), 16));
                            
                            if idx = N_SYSTEM - 1 then
                                idx <= 0;    
                            else
                                idx <= idx + 1;
                            end if;
                        end if;
                    end if;
                end process;
                
                LED <= led_reg;
            end Behavioral;
        "#},
        n = n,
        w = width,
        inputs_str = inputs_init_string
    );

    file.write_all(content.as_bytes())?;
    Ok(())
}
fn generate_comparator_file(width: usize) -> Result<()> {
    let mut file = File::create("vhdl/comparator.vhd")?;
    let content = format!(
        indoc! {r#"
            library IEEE;
            use IEEE.STD_LOGIC_1164.ALL;
            use IEEE.NUMERIC_STD.ALL;

            entity comparator is
                generic (
                    width : integer := {w}
                );
                Port (
                    in_A : in std_logic_vector(width - 1 downto 0);
                    in_B : in std_logic_vector(width - 1 downto 0);
                    dir  : in std_logic; -- '1' for ascending, '0' for descending
                    out_L : out std_logic_vector(width - 1 downto 0);
                    out_H : out std_logic_vector(width - 1 downto 0)
                );
            end comparator;

            architecture Behavioral of comparator is
            begin
                process(in_A, in_B, dir)
                begin
                
                    out_L <= in_A;
                    out_H <= in_B;

                    if (dir = '1' and unsigned(in_A) > unsigned(in_B)) then 
                        out_L <= in_B;
                        out_H <= in_A;
                        
                    elsif (dir = '0' and unsigned(in_A) < unsigned(in_B)) then 
                        out_L <= in_B;
                        out_H <= in_A;
                    end if;
                    
                end process;
            end Behavioral;
        "#},
        w = width
    );

    file.write_all(content.as_bytes())?;
    Ok(())
}
fn main() -> Result<()> {
    let n = 8;
    let data_width = 16;

    let file_network = File::create("vhdl/bitonic_network.vhd")?;
    let mut vhdl_writer = VhdlWriter::new(n, data_width, &file_network);
    
    vhdl_writer.write_start()?;
    vhdl_writer.write_wires()?;
    bitonic_sort_helper(&mut vhdl_writer, n as usize, 0, 1)?;
    vhdl_writer.write_end()?;
    println!("-> 'bitonic_network.vhd' generated.");

    generate_sorter_file(n, data_width)?;
    println!("-> 'sorter.vhd' generated.");

    generate_comparator_file(data_width)?;
    println!("-> 'comparator.vhd' generated.");
    Ok(())
}
