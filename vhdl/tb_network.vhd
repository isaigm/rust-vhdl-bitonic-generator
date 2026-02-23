library IEEE;
use IEEE.STD_LOGIC_1164.ALL;
use IEEE.NUMERIC_STD.ALL;
use work.network_types.all;

entity tb_network is
end entity tb_network;

architecture testbench of tb_network is

    constant N_TB     : integer := 8;
    constant WIDTH_TB : integer := 16;
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

        report "Waiting for pipeline propagation (6 total stages)...";
        for k in 1 to 3 loop
            wait until rising_edge(tb_clk);
        end loop;

        wait for 1 ns;
        if check_sorted_series(tb_outputs, 0) then
            report "--> SUCCESS [Cycle 6]: Series A correctly detected at output.";
        else
            report "FAILURE [Cycle 6]: Expected Series A." severity error;
        end if;

        wait until rising_edge(tb_clk);
        wait for 1 ns;
        if check_sorted_series(tb_outputs, 1) then
            report "--> SUCCESS [Cycle 7]: Series B correctly detected (1 cycle later).";
        else
            report "FAILURE [Cycle 7]: Expected Series B." severity error;
        end if;

        wait until rising_edge(tb_clk);
        wait for 1 ns;
        if check_sorted_series(tb_outputs, 2) then
            report "--> SUCCESS [Cycle 8]: Series C correctly detected (1 cycle later).";
        else
            report "FAILURE [Cycle 8]: Expected Series C." severity error;
        end if;

        report "--- END OF TEST ---";
        stop_clk <= true;
        wait;

    end process;
end architecture testbench;
