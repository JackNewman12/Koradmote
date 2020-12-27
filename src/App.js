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

function createData(name, calories, fat, carbs, protein) {
  return { name, calories, fat, carbs, protein };
}

const rows = [
  createData('Frozen yoghurt', 159, 6.0, 24, 4.0),
  createData('Ice cream sandwich', 237, 9.0, 37, 4.3),
  createData('Eclair', 262, 16.0, 24, 6.0),
  createData('Cupcake', 305, 3.7, 67, 4.3),
  createData('Gingerbread', 356, 16.0, 49, 3.9),
];

function PowerButton(props) {
  let [isDisabled, setisDisabled] = useState(false);
  let [isPowered, setisPowered] = useState(props.PowerState);

  let Clicked = () => {
    setisDisabled(true);
    console.log(`Toggling ${props.DevName}`);
    setTimeout(() => { setisDisabled(false); setisPowered(!isPowered)}, 2000);
  };

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
              <TableCell style={{fontWeight:900}}>Dessert (100g serving)</TableCell>
              <TableCell style={{fontWeight:900}}>Calories</TableCell>
              <TableCell style={{fontWeight:900}}>Fat&nbsp;(g)</TableCell>
              <TableCell style={{fontWeight:900}}>Carbs&nbsp;(g)</TableCell>
              <TableCell style={{fontWeight:900}}>Protein&nbsp;(g)</TableCell>
              <TableCell style={{fontWeight:900}}>Toggle Power</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row) => (
              <TableRow key={row.name}>
                <TableCell component="th" scope="row">
                  {row.name}
                </TableCell>
                <TableCell>{row.calories}</TableCell>
                <TableCell>{row.fat}</TableCell>
                <TableCell>{row.carbs}</TableCell>
                <TableCell>{row.protein}</TableCell>
                <TableCell><PowerButton DevName={row.name} PowerState={false}></PowerButton></TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}

export default App;
