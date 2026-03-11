use rand::Rng;
use std::fs::{self, File};
use std::io::{Write, BufWriter, Result};
use indoc::indoc;

struct VhdlWriter<'a> {
    id: usize,
    n: usize,
    data_width: usize,
    wires: Vec<String>,
    intermediate_regs: Vec<String>,
    inputs: Vec<String>,
    writer: BufWriter<&'a File>,
}

impl<'a> VhdlWriter<'a> {
    fn new(n: usize, data_width: usize, file: &'a File) -> Self {
        let log_n = (n as u32).ilog2();
        let num_stages = (log_n * (log_n + 1)) / 2;
        let total_comparators = (n * num_stages as usize) / 2;

        let mut wires: Vec<String> = vec![];
        let mut intermediate_regs: Vec<String> = vec![];

        for i in 0..total_comparators {
            wires.push(format!("w_l_{i}"));
            wires.push(format!("w_h_{i}"));
            intermediate_regs.push(format!("ir_l_{i}"));
            intermediate_regs.push(format!("ir_h_{i}"));
        }
        let mut inputs: Vec<String> = vec![];
        for i in 0..n {
            inputs.push(format!("inputs({})", i));
        }
        VhdlWriter {
            id: 0,
            n,
            data_width,
            wires,
            intermediate_regs,
            inputs,
            writer: BufWriter::new(file),
        }
    }
    fn write_comparator(
        writer: &mut BufWriter<&'a File>,
        id: usize,
        dir: u32,
        in_a: &str,
        in_b: &str,
        out_l: &str,
        out_h: &str
    ) -> Result<()> {
        writeln!(
            writer,
            "    comp_{id}: entity work.comparator port map(in_A => {in_a}, in_B => {in_b}, dir => '{dir}', out_L => {out_l}, out_H => {out_h});",
            id = id, in_a = in_a, in_b = in_b, dir = dir, out_l = out_l, out_h = out_h
        )?;
        Ok(())
    }

    fn write_wires(&mut self) -> Result<()> {
        for w in &self.wires {
            writeln!(self.writer, "    signal {} : std_logic_vector(width - 1 downto 0);", w)?;
        }
        for r in &self.intermediate_regs {
            writeln!(self.writer, "    signal {} : std_logic_vector(width - 1 downto 0);", r)?;
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
                        clk: in std_logic;
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
        let mut assign_statements: Vec<String> = vec![];
        for (i, wire) in self.wires.iter().enumerate() {
            assign_statements.push(format!("{} <= {};", self.intermediate_regs[i], wire ));
        }
        let as_statements_init_string = assign_statements.join("\n        ");

        let process = format!( indoc!{r#"
                process(clk)
                begin
                    if rising_edge(clk) then
                        {statements}
                    end if;
                end process;
        "#},statements = as_statements_init_string);
        for (i, output) in self.inputs.iter().enumerate() {
            writeln!(self.writer,"    outputs_array({}) <= {};", i, output)?;
        }
        self.writer.write_all(process.as_bytes())?;
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
            let id = vhdl_writer.id;
            let out_l = format!("w_l_{}", id);
            let out_h = format!("w_h_{}", id);
            let in_a = &vhdl_writer.inputs[i];
            let in_b = &vhdl_writer.inputs[i + k];
            VhdlWriter::write_comparator(
                &mut vhdl_writer.writer,
                id,
                dir,
                in_a,
                in_b,
                &out_l,
                &out_h
            )?;
            vhdl_writer.inputs[i] = format!("ir_l_{}", id);
            vhdl_writer.inputs[i + k] = format!("ir_h_{}", id);
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
        let num = rng.random_range(0..=9999);
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
                    seg: out std_logic_vector(6 downto 0);
                    an: out std_logic_vector(3 downto 0);
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
                signal start: std_logic := '0';
                signal output_number: std_logic_vector(15 downto 0) := (others => '0');
                signal ready: std_logic := '0';
                signal reset: std_logic := '0';

            begin
                network: entity work.bitonic_network
                generic map (
                    n => N_SYSTEM,
                    width => W_SYSTEM
                )
                port map(
                    clk => CLK100MHZ,
                    inputs => inputs,
                    outputs => outputs
                );

                push_btn: entity work.push_btn port map(clk => CLK100MHZ, btn => btnR, enabled => enabled);
                binary_to_bcd: entity work.binary_to_bcd port map(clk => CLK100MHZ,
                reset => reset,
                start => start,
                input_number => led_reg,
                output_number => output_number,
                ready => ready);
                display: entity work.seven_segment_display port map(clk => CLK100MHZ,
                number => output_number, seg => seg, an => an );
                process (CLK100MHZ)
                begin
                    if rising_edge(CLK100MHZ) then
                        if enabled = '1' then
                            start <= '1';
                            led_reg <= std_logic_vector(resize(unsigned(outputs(idx)), 16));

                            if idx = N_SYSTEM - 1 then
                                idx <= 0;
                            else
                                idx <= idx + 1;
                            end if;
                        else
                            start <= '0';
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
    let symbols : [[char; 2]; 2] = [['<', '>'], ['>', '<']];
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

// NUEVA FUNCIÓN: Genera el Testbench dinámicamente según N
fn generate_testbench_file(n: usize, width: usize) -> Result<()> {
    let mut file = File::create("vhdl/tb_network.vhd")?;

    // Calcular la latencia exacta matemáticamente
    let log_n = (n as u32).ilog2();
    let num_stages = (log_n * (log_n + 1)) / 2;

    // Como inyectamos 3 series (T=0, T=1, T=2), gastamos 3 ciclos.
    // El testbench debe esperar el resto de los ciclos.
    let remaining_wait = if num_stages > 3 { num_stages - 3 } else { 0 };

    let content = format!(
        indoc! {r#"
            library IEEE;
            use IEEE.STD_LOGIC_1164.ALL;
            use IEEE.NUMERIC_STD.ALL;
            use work.network_types.all;

            entity tb_network is
            end entity tb_network;

            architecture testbench of tb_network is

                constant N_TB     : integer := {n};
                constant WIDTH_TB : integer := {w};
                constant CLK_PERIOD : time := 10 ns;

                component bitonic_network is
                    generic ( n : integer; width : integer );
                    Port ( clk : in std_logic; inputs : in mem; outputs : out mem );
                end component;

                signal tb_clk     : std_logic := '0';
                signal tb_inputs  : mem(0 to N_TB - 1)(WIDTH_TB - 1 downto 0) := (others => (others => '0'));
                signal tb_outputs : mem(0 to N_TB - 1)(WIDTH_TB - 1 downto 0);
                signal stop_clk   : boolean := false;

                function check_sorted_series(current_out : mem; offset : integer) return boolean is
                    variable val : integer;
                begin
                    for i in 0 to N_TB - 1 loop
                        val := (i + 1) * 10 + offset;
                        if current_out(i) /= std_logic_vector(to_unsigned(val, WIDTH_TB)) then
                            return false;
                        end if;
                    end loop;
                    return true;
                end function;

            begin

                UUT: bitonic_network
                    generic map (n => N_TB, width => WIDTH_TB)
                    port map (clk => tb_clk, inputs => tb_inputs, outputs => tb_outputs);

                clk_process: process
                begin
                    while not stop_clk loop
                        tb_clk <= '0'; wait for CLK_PERIOD / 2;
                        tb_clk <= '1'; wait for CLK_PERIOD / 2;
                    end loop;
                    wait;
                end process;

                stimulus_process: process
                begin
                    report "--- START OF THROUGHPUT TEST (PIPELINE) ---";

                    tb_inputs <= (others => (others => '0'));
                    wait for 5 * CLK_PERIOD;
                    wait until falling_edge(tb_clk);

                    report "Input T=0: Sending Series A (ending in 0)";
                    for i in 0 to N_TB - 1 loop
                        tb_inputs(i) <= std_logic_vector(to_unsigned((N_TB - i) * 10, WIDTH_TB));
                    end loop;
                    wait until rising_edge(tb_clk);

                    report "Input T=1: Sending Series B (ending in 1)";
                    for i in 0 to N_TB - 1 loop
                        tb_inputs(i) <= std_logic_vector(to_unsigned((N_TB - i) * 10 + 1, WIDTH_TB));
                    end loop;
                    wait until rising_edge(tb_clk);

                    report "Input T=2: Sending Series C (ending in 2)";
                    for i in 0 to N_TB - 1 loop
                        tb_inputs(i) <= std_logic_vector(to_unsigned((N_TB - i) * 10 + 2, WIDTH_TB));
                    end loop;
                    wait until rising_edge(tb_clk);

                    tb_inputs <= (others => (others => '0'));

                    report "Waiting for pipeline propagation ({total_latency} total stages)...";
                    for k in 1 to {remaining_wait} loop
                        wait until rising_edge(tb_clk);
                    end loop;

                    wait for 1 ns;
                    if check_sorted_series(tb_outputs, 0) then
                        report "--> SUCCESS [Cycle {total_latency}]: Series A correctly detected at output.";
                    else
                        report "FAILURE [Cycle {total_latency}]: Expected Series A." severity error;
                    end if;

                    wait until rising_edge(tb_clk);
                    wait for 1 ns;
                    if check_sorted_series(tb_outputs, 1) then
                        report "--> SUCCESS [Cycle {total_latency_plus_1}]: Series B correctly detected (1 cycle later).";
                    else
                        report "FAILURE [Cycle {total_latency_plus_1}]: Expected Series B." severity error;
                    end if;

                    wait until rising_edge(tb_clk);
                    wait for 1 ns;
                    if check_sorted_series(tb_outputs, 2) then
                        report "--> SUCCESS [Cycle {total_latency_plus_2}]: Series C correctly detected (1 cycle later).";
                    else
                        report "FAILURE [Cycle {total_latency_plus_2}]: Expected Series C." severity error;
                    end if;

                    report "--- END OF TEST ---";
                    stop_clk <= true;
                    wait;

                end process;
            end architecture testbench;
        "#},
        n = n,
        w = width,
        total_latency = num_stages,
        remaining_wait = remaining_wait,
        total_latency_plus_1 = num_stages + 1,
        total_latency_plus_2 = num_stages + 2
    );

    file.write_all(content.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let n = 8; 
    let data_width = 16;

    fs::create_dir_all("vhdl")?;

    let file_network = File::create("vhdl/bitonic_network.vhd")?;
    let mut vhdl_writer = VhdlWriter::new(n, data_width, &file_network);

    vhdl_writer.write_start()?;
    vhdl_writer.write_wires()?;
    bitonic_sort_helper(&mut vhdl_writer, n as usize, 0, 0)?;
    vhdl_writer.write_end()?;
    println!("-> 'bitonic_network.vhd' generated.");

    generate_sorter_file(n, data_width)?;
    println!("-> 'sorter.vhd' generated.");

    generate_comparator_file(data_width)?;
    println!("-> 'comparator.vhd' generated.");

    // Generamos el testbench dinámico
    generate_testbench_file(n, data_width)?;
    println!("-> 'tb_network.vhd' generated.");

    Ok(())
}
