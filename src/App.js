import logo from './logo.svg';
import Button from '@material-ui/core/Button'
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import TableContainer from '@material-ui/core/TableContainer';
import Paper from '@material-ui/core/Paper';
import { useState, useEffect } from 'react';
import AppBar from '@material-ui/core/AppBar';
import { CircularProgress, Toolbar } from '@material-ui/core';
import './App.css';

function createData(name, voltage, current, state) {
  return { name, voltage, current, state };
}

function PowerButton(props) {
  let [isDisabled, setisDisabled] = useState(false);
  let [isPowered, setisPowered] = useState(props.PowerState);

  let Clicked = () => {
    setisDisabled(true);
    console.log(`Toggling ${props.DevName}`);
    setTimeout(HandleClickData, 300);
  };

  let HandleClickData = () => {
    setisDisabled(false);
    setisPowered(!isPowered);
  }

  useEffect(() => {
    setisPowered(props.PowerState);
  }, [props]);

  return <Button variant="contained"
    style={{ backgroundColor: isPowered ? "limegreen" : "red" }}
    disabled={isDisabled}
    onClick={Clicked} >
    {isPowered ? "On" : "Off"}
  </Button>
}

function App() {

  let [rows, setrowsdata] = useState([]);
  let [pendingData, setpendingData] = useState(false);

  function updateData(myRows) {
    setrowsdata(myRows);
    setpendingData(false);
  }

  useEffect(() => {

    function requestUpdate() {
      setpendingData(true);
      const myRows = [
        createData('Device1', Math.random() + 11.5, Math.random(), Math.random() > 0.5),
        createData('Device2', Math.random() + 11.5, Math.random(), false),
        createData('Device3', Math.random() + 11.5, Math.random(), true),
      ];

      setTimeout(() => { updateData(myRows) }, 400);
    }

    requestUpdate();
    const interval = setInterval(requestUpdate, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="App">
      <AppBar >
        <Toolbar >
          <img src={logo} className="App-logo" alt="logo" height={40} />
          Power Supply Thingo
          {pendingData && <CircularProgress disableShrink color="secondary" style={{ "marginLeft": "10px" }} />}
          {/* <Fab color="secondary" aria-label="add" style={{ margin: 0, right: 20, position: 'fixed' }}
          onClick={updateData}>
            <RefreshIcon />
          </Fab> */}
        </Toolbar>
      </AppBar>
      <Toolbar />
      <div className="Table">
        <TableContainer component={Paper}>
          <Table className={"JackTest"} aria-label="simple table">
            <TableHead>
              <TableRow>
                <TableCell style={{ fontWeight: 900 }}>Name</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Voltage</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Current</TableCell>
                <TableCell style={{ fontWeight: 900 }}>Current State</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {rows.map((row) => (
                <TableRow key={row.name}>
                  <TableCell component="th" scope="row">
                    {row.name}
                  </TableCell>
                  <TableCell>{row.voltage.toFixed(2)}</TableCell>
                  <TableCell>{row.current.toFixed(2)}</TableCell>
                  <TableCell><PowerButton DevName={row.name} PowerState={row.state}></PowerButton></TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </div>
    </div>
  );
}

export default App;
