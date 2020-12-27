import logo from './logo.svg';
import Button from '@material-ui/core/Button'
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Fab from '@material-ui/core/Fab';
import RefreshIcon from '@material-ui/icons/Refresh';
import { useState } from 'react';
import AppBar from '@material-ui/core/AppBar';
import { Toolbar } from '@material-ui/core';
import './App.css';

function createData(name, voltage, current, state) {
  return { name, voltage, current, state };
}

const rows = [
  createData('Device1', 12.1, 1.2, true),
  createData('Device2', 12.2, 1.6, false),
  createData('Device3', 12.3, 1.3, true),
];

function PowerButton(props) {
  let [isDisabled, setisDisabled] = useState(false);
  let [isPowered, setisPowered] = useState(props.PowerState);

  let Clicked = () => {
    setisDisabled(true);
    console.log(`Toggling ${props.DevName}`);
    setTimeout(HandleClickData, 2000);
  };

  let HandleClickData = () => {
    setisDisabled(false);
    setisPowered(!isPowered);
  }

  return <Button variant="contained" style={{backgroundColor:isPowered? "green":"red"}} disabled={isDisabled} onClick={Clicked} >
    {isDisabled ? "Loading" : (isPowered ? "On" : "Off")}</Button>
}

function App() {

  return (
    <div className="App">
      <AppBar >
        <Toolbar >
          <img src={logo} className="App-logo" alt="logo" height={40} />
          Power Supply Thingo
          <Fab color="secondary" aria-label="add" style={{ margin: 0, right: 20, position: 'fixed' }}>
            <RefreshIcon />
          </Fab>
        </Toolbar>
      </AppBar>
      <Toolbar />
      <div>
        <Table className={"JackTest"} aria-label="simple table" style={{backgroundColor: "white" }}>
          <TableHead>
            <TableRow>
              <TableCell style={{fontWeight:900}}>Name</TableCell>
              <TableCell style={{fontWeight:900}}>Voltage</TableCell>
              <TableCell style={{fontWeight:900}}>Current</TableCell>
              <TableCell style={{fontWeight:900}}>Toggle Power</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.name}>
                <TableCell component="th" scope="row">
                  {row.name}
                </TableCell>
                <TableCell>{row.voltage}</TableCell>
                <TableCell>{row.current}</TableCell>
                <TableCell><PowerButton DevName={row.name} PowerState={row.state}></PowerButton></TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

export default App;
